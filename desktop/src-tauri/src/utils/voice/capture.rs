use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use whisper_rs::{FullParams, SamplingStrategy};

use crate::utils::voice::dispatcher;
use crate::utils::voice::session::SharedSession;
use crate::utils::voice::{
    audio_processor, command_resolver, speaker, state::get_whisper, wake_word,
};

const TARGET_RATE: u32 = 16000;
const FLUSH_SECS: u64 = 5;

pub struct CaptureHandle {
    _stream: cpal::Stream,
}

pub fn start(app: AppHandle, session: SharedSession) -> Result<CaptureHandle, String> {
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

    let native_chunk = native_rate as usize * FLUSH_SECS as usize;

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(FLUSH_SECS));

        let chunk: Vec<f32> = {
            let mut buf = buffer.lock().unwrap();
            if buf.len() < native_chunk / 4 {
                eprintln!("[capture] buffer too small, skipping");
                continue;
            }
            buf.drain(..).collect()
        };

        let resampled = resample(chunk, native_rate, TARGET_RATE);
        eprintln!("[capture] resampled to {} samples", resampled.len());

        let processed = audio_processor::process_f32(resampled.clone());
        if processed.is_empty() {
            eprintln!("[capture] silence — skipping");
            emit(
                &app_flush,
                VoiceEvent {
                    transcript: String::new(),
                    wake_detected: false,
                    command: None,
                    speaker_identified: None,
                },
            );
            continue;
        }

        let transcript = transcribe(&processed);
        eprintln!("[capture] transcript: {:?}", transcript);

        if transcript.is_empty() {
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

        let wake = wake_word::detect(&transcript);
        eprintln!("[capture] wake={}", wake.wake_detected);

        if wake.wake_detected {
            // Wake word = identify who is speaking, fill session, nothing else
            let emb = speaker::extract_embedding(&resampled);
            let speaker_name = emb.as_ref().map(|e| {
                eprintln!(
                    "[capture] wake detected — identifying speaker (dim={})",
                    e.len()
                );
                e.clone()
            });

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
                    speaker_identified: speaker_name,
                },
            );
        } else {
            // No wake word — try to match a command directly from transcript
            let resolved = command_resolver::resolve(&transcript);

            let command = if resolved.intent != "chat" && resolved.intent != "wake_only" {
                // Only dispatch if we actually matched something meaningful
                let resolved_clone = resolved.clone();
                let session_clone = session.clone();
                tauri::async_runtime::spawn(async move {
                    dispatcher::dispatch(resolved_clone, session_clone).await;
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
    /// Raw embedding sent to frontend when wake word fires — JS can display who was identified
    pub speaker_identified: Option<Vec<f32>>,
}

fn emit(app: &AppHandle, event: VoiceEvent) {
    if let Err(e) = app.emit("peppa://voice-result", event) {
        eprintln!("[capture] emit error: {e}");
    }
}
