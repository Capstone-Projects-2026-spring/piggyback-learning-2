use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use whisper_rs::{FullParams, SamplingStrategy};

use super::enrollment::{create_user, emit_enrollment, EnrollmentEvent};
use super::onboarding::{
    average_embeddings, begin_voice_collection, record_embedding, OnboardingStage,
};
use super::vad::VadChunker;
use crate::utils::voice::{
    audio_processor, command_resolver, dispatcher,
    onboarding::{self, SharedOnboarding},
    session::SharedSession,
    speaker,
    state::get_whisper,
    wake_word,
};

const TARGET_RATE: u32 = 16000;

pub struct CaptureHandle {
    _stream: cpal::Stream,
}

pub fn start(
    app: AppHandle,
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
    let buffer_w = buffer.clone();
    let app_flush = app.clone();
    let err_fn = |e| eprintln!("[capture] stream error: {e}");

    let stream = match supported.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| push_samples(data, channels, &buffer_w),
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                let f: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                push_samples(&f, channels, &buffer_w);
            },
            err_fn,
            None,
        ),
        SampleFormat::U8 => device.build_input_stream(
            &config,
            move |data: &[u8], _| {
                let f: Vec<f32> = data.iter().map(|&s| (s as f32 - 128.0) / 128.0).collect();
                push_samples(&f, channels, &buffer_w);
            },
            err_fn,
            None,
        ),
        fmt => return Err(format!("[capture] unsupported format: {fmt}")),
    }
    .map_err(|e| format!("[capture] build_input_stream: {e}"))?;

    stream.play().map_err(|e| format!("[capture] play: {e}"))?;
    eprintln!(
        "[capture] stream started at {}Hz {} ch",
        native_rate, channels
    );

    std::thread::spawn(move || {
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

            let resampled = resample(raw, native_rate, TARGET_RATE);

            let Some(chunk) = chunker.push(&resampled) else {
                continue;
            };

            eprintln!(
                "[capture] utterance ready: {} samples ({:.1}s)",
                chunk.len(),
                chunk.len() as f32 / TARGET_RATE as f32
            );

            let processed = audio_processor::process_f32(chunk.clone());
            if processed.is_empty() {
                eprintln!("[capture] silence after processing — skipping");
                continue;
            }

            let transcript = transcribe(&processed);
            eprintln!("[capture] transcript: {:?}", transcript);
            if transcript.is_empty() {
                continue;
            }

            // Onboarding takes priority over normal pipeline
            {
                let o = onboarding.lock().unwrap();
                if o.is_active() {
                    drop(o);
                    handle_onboarding_audio(&app_flush, &onboarding, &session, &chunk, &transcript);
                    continue;
                }
            }
            {
                let mut s = session.lock().unwrap();
                if s.mode == crate::utils::voice::session::SessionMode::Answer {
                    s.last_transcript = Some(transcript.clone());
                    drop(s);

                    let sc = session.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::handlers::answers::analyze_answer(&[], &sc).await;
                    });

                    emit(
                        &app_flush,
                        VoiceEvent {
                            transcript,
                            wake_detected: false,
                            command: None,
                            speaker_identified: None,
                        },
                    );
                    continue;
                }
            }
            if is_noise_transcript(&transcript) {
                eprintln!("[capture] transcript looks like noise — skipping");
                continue;
            }

            let wake = wake_word::detect(&transcript);
            eprintln!("[capture] wake={}", wake.wake_detected);

            if wake.wake_detected {
                let emb = speaker::extract_embedding(&chunk);
                let speaker_identified = emb.as_ref().map(|e| e.clone());

                if let Some(emb) = emb {
                    let session_clone = session.clone();
                    tauri::async_runtime::spawn(async move {
                        speaker::identify_speaker(&emb, &session_clone).await;
                    });
                }

                emit(
                    &app_flush,
                    VoiceEvent {
                        transcript,
                        wake_detected: true,
                        command: None,
                        speaker_identified,
                    },
                );
            } else {
                let resolved = command_resolver::resolve(&transcript);
                let command = if resolved.intent != "chat" && resolved.intent != "wake_only" {
                    let rc = resolved.clone();
                    let sc = session.clone();
                    let oc = onboarding.clone();
                    let ac = app_flush.clone();
                    tauri::async_runtime::spawn(async move {
                        dispatcher::dispatch(ac, rc, sc, oc).await;
                    });
                    Some(resolved)
                } else {
                    eprintln!("[capture] no command matched — passive listen");
                    None
                };

                emit(
                    &app_flush,
                    VoiceEvent {
                        transcript,
                        wake_detected: false,
                        command,
                        speaker_identified: None,
                    },
                );
            }
        }
    });

    Ok(CaptureHandle { _stream: stream })
}

fn push_samples(data: &[f32], channels: usize, buffer: &Arc<Mutex<Vec<f32>>>) {
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();
    buffer.lock().unwrap().extend_from_slice(&mono);
}

fn resample(samples: Vec<f32>, from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return samples;
    }
    let ratio = from as f64 / to as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = pos as usize;
            let frac = (pos - idx as f64) as f32;
            let s0 = samples[idx.min(samples.len() - 1)];
            let s1 = samples[(idx + 1).min(samples.len() - 1)];
            s0 + (s1 - s0) * frac
        })
        .collect()
}

const NOISE_TRANSCRIPTS: &[&str] = &[
    "you",
    "the",
    "a",
    "uh",
    "um",
    "oh",
    "ah",
    "hm",
    "hmm",
    "thank you",
    "thanks",
    "bye",
    "okay",
    "ok",
];

fn is_noise_transcript(transcript: &str) -> bool {
    let t = transcript.trim().to_lowercase();
    // Single word that's just a filler
    if t.split_whitespace().count() == 1 && NOISE_TRANSCRIPTS.contains(&t.as_str()) {
        return true;
    }
    // Very short — under 3 chars after trimming punctuation
    let alpha: String = t.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha.len() < 3 {
        return true;
    }
    false
}

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

#[derive(serde::Serialize, Clone)]
pub struct VoiceEvent {
    pub transcript: String,
    pub wake_detected: bool,
    pub command: Option<crate::utils::voice::command_resolver::ResolvedCommand>,
    pub speaker_identified: Option<Vec<f32>>,
}

fn emit(app: &AppHandle, event: VoiceEvent) {
    if let Err(e) = app.emit("peppa://voice-result", event) {
        eprintln!("[capture] emit error: {e}");
    }
}

fn handle_onboarding_audio(
    app: &AppHandle,
    onboarding: &SharedOnboarding,
    session: &SharedSession,
    audio: &[f32],
    transcript: &str,
) {
    let stage = onboarding.lock().unwrap().stage.clone();

    match stage {
        OnboardingStage::WaitingForName => {
            let accepted = onboarding::try_set_name(app, onboarding, transcript);
            if !accepted {
                eprintln!("[onboarding] name rejected, still waiting");
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            begin_voice_collection(app, onboarding);
        }

        OnboardingStage::CollectingVoice { prompt_index } => {
            eprintln!(
                "[onboarding] collecting prompt {prompt_index}, audio_len={}",
                audio.len()
            );

            let embedding = match speaker::extract_embedding(audio) {
                Some(e) => e,
                None => {
                    eprintln!("[onboarding] no embedding — waiting for next chunk");
                    return;
                }
            };

            eprintln!("[onboarding] embedding dim={}", embedding.len());
            let done = record_embedding(app, onboarding, embedding);

            if done {
                let o = onboarding.lock().unwrap();
                let name = o.name.clone().unwrap_or("User".to_string());
                let role = o.flow.role().to_string();
                let avg = average_embeddings(&o.embeddings);
                drop(o);

                let app_clone = app.clone();
                let session_clone = session.clone();

                tauri::async_runtime::spawn(async move {
                    match create_user(name.clone(), avg, &role).await {
                        Ok(id) => {
                            // Only populate session for parent — kid enrollment is
                            // initiated by an already-identified parent so we leave
                            // the session as-is after kid creation.
                            if role == "parent" {
                                session_clone.lock().unwrap().set_user(
                                    id,
                                    name.clone(),
                                    role.clone(),
                                );
                            }
                            emit_enrollment(
                                &app_clone,
                                EnrollmentEvent {
                                    stage: if role == "parent" {
                                        "done".to_string()
                                    } else {
                                        "kid_done".to_string()
                                    },
                                    message: if role == "parent" {
                                        format!(
                                            "You're all set, {name}! \
                                             Say 'hey Peppa' whenever you need me!"
                                        )
                                    } else {
                                        format!(
                                            "You're all set, {name}! \
                                             Say 'hey Peppa' whenever you want to learn something!"
                                        )
                                    },
                                    prompt_index: 0,
                                    total_prompts: 0,
                                    prompts: vec![],
                                    flow: role.clone(),
                                },
                            );
                        }
                        Err(e) => {
                            eprintln!("[onboarding] create_user failed: {e}");
                            emit_enrollment(
                                &app_clone,
                                EnrollmentEvent {
                                    stage: "error".to_string(),
                                    message: "Something went wrong. Please restart.".to_string(),
                                    prompt_index: 0,
                                    total_prompts: 0,
                                    prompts: vec![],
                                    flow: role,
                                },
                            );
                        }
                    }
                });
            }
        }

        _ => {}
    }
}
