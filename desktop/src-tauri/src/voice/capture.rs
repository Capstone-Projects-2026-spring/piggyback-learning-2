use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};

use super::enrollment::{create_user, emit_enrollment, EnrollmentEvent};
use super::onboarding::{begin_voice_collection, record_embedding, OnboardingStage};
use super::vad::VadChunker;
use crate::utils::{app_handle, text::is_noise_transcript};
use crate::voice::{
    audio_processor, command_resolver,
    command_resolver::ResolvedCommand,
    dispatcher,
    intent::Intent,
    moonshine,
    onboarding::{self, SharedOnboarding},
    session::{SessionMode, SharedSession},
    speaker, wake_word,
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

fn run_capture_loop(
    session: SharedSession,
    onboarding: SharedOnboarding,
    buffer: Arc<Mutex<Vec<f32>>>,
    native_rate: u32,
) {
    let mut chunker = VadChunker::new();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(200));

        // TTS gate - while TTS is active, drain and discard everything.
        // This is the single source of truth. Nothing processes while TTS plays.
        {
            let s = session.lock().unwrap();
            if s.mode == SessionMode::Tts {
                drop(s);
                buffer.lock().unwrap().clear();
                chunker.flush("tts active");
                continue;
            }
        }

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

        // Re-check after VAD - TTS may have started while accumulating.
        {
            let s = session.lock().unwrap();
            if s.mode == SessionMode::Tts {
                eprintln!("[capture] TTS mode post-VAD - dropping chunk");
                chunker.flush("tts active post-vad");
                continue;
            }
        }

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

        // Re-check after transcription - Moonshine takes 1-3s.
        {
            let s = session.lock().unwrap();
            if s.mode == SessionMode::Tts {
                eprintln!("[capture] TTS mode post-transcription - dropping");
                continue;
            }
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

fn transcribe(samples: &[f32]) -> String {
    moonshine::transcribe(samples)
}

#[derive(serde::Serialize, Clone)]
pub struct VoiceEvent {
    pub transcript: String,
    pub wake_detected: bool,
    pub command: Option<ResolvedCommand>,
}

fn emit_voice(event: VoiceEvent) {
    app_handle::emit("orb://voice-result", event);
}

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
            // Spawn so the capture thread returns immediately and keeps
            // hitting the TTS gate at the top of the loop.
            let onboarding = onboarding.clone();
            std::thread::spawn(move || {
                begin_voice_collection(&onboarding);
            });
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
                let embeddings = o.embeddings.clone();
                drop(o);

                let session_clone = session.clone();

                tauri::async_runtime::spawn(async move {
                    match create_user(name.clone(), embeddings, &role).await {
                        Ok(id) => {
                            if role == "parent" {
                                session_clone.lock().unwrap().set_user(
                                    id,
                                    name.clone(),
                                    role.clone(),
                                );
                            }

                            let (stage, message) = if role == "parent" {
                                ("done", format!("You're all set, {name}! Say 'Hey Jarvis' whenever you need me!"))
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
            // If not done: record_embedding already emitted the next prompt
            // which triggers TTS on the frontend. The capture loop will
            // discard everything via the TTS gate until TTS finishes.
        }

        other => eprintln!("[onboarding] unhandled stage: {other:?}"),
    }
}
