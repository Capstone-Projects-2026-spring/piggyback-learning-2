use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy};

use super::enrollment::{create_user, emit_enrollment, EnrollmentEvent};
use super::onboarding::{
    average_embeddings, begin_voice_collection, record_embedding, OnboardingStage,
};
use super::vad::VadChunker;
use crate::utils::{app_handle, text::is_noise_transcript};
use crate::voice::{
    audio_processor, command_resolver,
    command_resolver::ResolvedCommand,
    dispatcher,
    intent::Intent,
    onboarding::{self, SharedOnboarding},
    session::{SessionMode, SharedSession},
    speaker,
    state::get_whisper,
    wake_word,
};

const TARGET_RATE: u32 = 16000;

pub struct CaptureHandle {
    _stream: cpal::Stream,
}

pub fn start(
    session: SharedSession,
    onboarding: SharedOnboarding,
) -> Result<CaptureHandle, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("[capture] no input device found")?;

    eprintln!("[capture] device: {}", device.name().unwrap_or_default());

    let supported = device
        .default_input_config()
        .map_err(|e| format!("[capture] default_input_config: {e}"))?;

    eprintln!("[capture] native config: {:?}", supported);

    let native_rate = supported.sample_rate().0;
    let channels = supported.channels() as usize;

    let config = StreamConfig {
        channels: supported.channels(),
        sample_rate: SampleRate(native_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = {
        let buffer_w = buffer.clone();
        match supported.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mono = audio_processor::to_mono(data, channels);
                    buffer_w.lock().unwrap().extend_from_slice(&mono);
                },
                |e| eprintln!("[capture] stream error: {e}"),
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let mono = audio_processor::to_mono(&f, channels);
                    buffer_w.lock().unwrap().extend_from_slice(&mono);
                },
                |e| eprintln!("[capture] stream error: {e}"),
                None,
            ),
            SampleFormat::U8 => device.build_input_stream(
                &config,
                move |data: &[u8], _| {
                    let f: Vec<f32> = data.iter().map(|&s| (s as f32 - 128.0) / 128.0).collect();
                    let mono = audio_processor::to_mono(&f, channels);
                    buffer_w.lock().unwrap().extend_from_slice(&mono);
                },
                |e| eprintln!("[capture] stream error: {e}"),
                None,
            ),
            fmt => return Err(format!("[capture] unsupported sample format: {fmt}")),
        }
        .map_err(|e| format!("[capture] build_input_stream: {e}"))?
    };

    stream.play().map_err(|e| format!("[capture] play: {e}"))?;
    eprintln!(
        "[capture] stream started at {}Hz {}ch",
        native_rate, channels
    );

    std::thread::spawn(move || run_capture_loop(session, onboarding, buffer, native_rate));

    Ok(CaptureHandle { _stream: stream })
}

// Capture loop

fn run_capture_loop(
    session: SharedSession,
    onboarding: SharedOnboarding,
    buffer: Arc<Mutex<Vec<f32>>>,
    native_rate: u32,
) {
    let mut chunker = VadChunker::new();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(30));

        let raw: Vec<f32> = {
            let mut buf = buffer.lock().unwrap();
            buf.drain(..).collect()
        };

        if raw.is_empty() {
            continue;
        }

        let resampled = audio_processor::resample(raw, native_rate, TARGET_RATE);
        let Some(chunk) = chunker.push(&resampled) else {
            continue;
        };

        eprintln!(
            "[capture] utterance ready - {} samples ({:.1}s)",
            chunk.len(),
            chunk.len() as f32 / TARGET_RATE as f32
        );

        let processed = audio_processor::process_f32(chunk.clone());
        if processed.is_empty() {
            eprintln!("[capture] silence after processing - skipping");
            continue;
        }

        let transcript = transcribe(&processed);
        eprintln!("[capture] transcript: {:?}", transcript);
        if transcript.is_empty() {
            continue;
        }

        // Onboarding - highest priority
        {
            let o = onboarding.lock().unwrap();
            if o.is_active() {
                drop(o);
                handle_onboarding_audio(&onboarding, &session, &chunk, &transcript);
                continue;
            }
        }

        // Answer mode - bypass wake + classifier
        // Transcript is written into session before spawning so analyze_answer
        // reads a consistent snapshot even if another utterance arrives quickly.
        {
            let mut s = session.lock().unwrap();
            if s.mode == SessionMode::Answer {
                s.last_transcript = Some(transcript.clone());
                drop(s);

                let sc = session.clone();
                tauri::async_runtime::spawn(async move {
                    crate::handlers::answers::analyze_answer(&[], &sc).await;
                });

                emit_voice(VoiceEvent {
                    transcript,
                    wake_detected: false,
                    command: None,
                });
                continue;
            }
        }

        // Normal pipeline
        if is_noise_transcript(&transcript) {
            eprintln!("[capture] noise transcript - skipping");
            continue;
        }

        let wake = wake_word::detect(&transcript);

        if wake.wake_detected {
            // Speaker ID runs async - does not block the audio loop
            if let Some(emb) = speaker::extract_embedding(&chunk) {
                let sc = session.clone();
                tauri::async_runtime::spawn(async move {
                    speaker::identify_speaker(&emb, &sc).await;
                });
            }
            emit_voice(VoiceEvent {
                transcript,
                wake_detected: true,
                command: None,
            });
        } else {
            let resolved = command_resolver::resolve(&transcript);

            match resolved.intent {
                Intent::Unhandled | Intent::WakeOnly => {
                    eprintln!("[capture] no command matched - passive listen");
                    emit_voice(VoiceEvent {
                        transcript,
                        wake_detected: false,
                        command: None,
                    });
                }
                _ => {
                    let rc = resolved.clone();
                    let sc = session.clone();
                    let oc = onboarding.clone();
                    tauri::async_runtime::spawn(async move {
                        dispatcher::dispatch(rc, sc, oc).await;
                    });
                    emit_voice(VoiceEvent {
                        transcript,
                        wake_detected: false,
                        command: Some(resolved),
                    });
                }
            }
        }
    }
}

// Transcription

fn transcribe(samples: &[f32]) -> String {
    let ctx = get_whisper();
    let mut state = match ctx.create_state() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[capture] create_state: {e}");
            return String::new();
        }
    };

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    if let Err(e) = state.full(params, samples) {
        eprintln!("[capture] whisper full: {e}");
        return String::new();
    }

    let n = state.full_n_segments();
    (0..n)
        .filter_map(|i| state.get_segment(i).map(|s| s.to_string()))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase()
}

// Events

#[derive(serde::Serialize, Clone)]
pub struct VoiceEvent {
    pub transcript: String,
    pub wake_detected: bool,
    pub command: Option<ResolvedCommand>,
}

fn emit_voice(event: VoiceEvent) {
    app_handle::emit("orb://voice-result", event);
}

// Onboarding

fn handle_onboarding_audio(
    onboarding: &SharedOnboarding,
    session: &SharedSession,
    audio: &[f32],
    transcript: &str,
) {
    let stage = onboarding.lock().unwrap().stage.clone();

    match stage {
        OnboardingStage::WaitingForName => {
            if !onboarding::try_set_name(onboarding, transcript) {
                eprintln!("[onboarding] name rejected - waiting");
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            begin_voice_collection(onboarding);
        }

        OnboardingStage::CollectingVoice { prompt_index } => {
            eprintln!(
                "[onboarding] prompt {prompt_index} audio_len={}",
                audio.len()
            );

            let Some(embedding) = speaker::extract_embedding(audio) else {
                eprintln!("[onboarding] no embedding - waiting for next chunk");
                return;
            };

            eprintln!("[onboarding] embedding dim={}", embedding.len());
            let done = record_embedding(onboarding, embedding);

            if done {
                let o = onboarding.lock().unwrap();
                let name = o.name.clone().unwrap_or_else(|| "User".to_string());
                let role = o.flow.role().to_string();
                let avg = average_embeddings(&o.embeddings);
                drop(o);

                let session_clone = session.clone();

                tauri::async_runtime::spawn(async move {
                    match create_user(name.clone(), avg, &role).await {
                        Ok(id) => {
                            // Only update session for parent - kid enrollment is
                            // triggered by an already-identified parent so the
                            // session stays as-is after kid creation.
                            if role == "parent" {
                                session_clone.lock().unwrap().set_user(
                                    id,
                                    name.clone(),
                                    role.clone(),
                                );
                            }

                            let (stage, message) = if role == "parent" {
                                ("done",     format!("You're all set, {name}! Say 'Hey Jarvis' whenever you need me!"))
                            } else {
                                ("kid_done", format!("You're all set, {name}! Say 'Hey Jarvis' whenever you want to learn something!"))
                            };

                            emit_enrollment(EnrollmentEvent {
                                stage: stage.to_string(),
                                message,
                                prompt_index: 0,
                                total_prompts: 0,
                                prompts: vec![],
                                flow: role,
                            });
                        }
                        Err(e) => {
                            eprintln!("[onboarding] create_user failed: {e}");
                            emit_enrollment(EnrollmentEvent {
                                stage: "error".to_string(),
                                message: "Something went wrong. Please restart.".to_string(),
                                prompt_index: 0,
                                total_prompts: 0,
                                prompts: vec![],
                                flow: role,
                            });
                        }
                    }
                });
            }
        }

        other => eprintln!("[onboarding] unhandled stage: {other:?}"),
    }
}
