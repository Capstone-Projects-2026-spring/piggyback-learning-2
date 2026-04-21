use crate::db::init::{get_db, get_voice_key};
use crate::utils::crypto;
use crate::utils::voice::session::SharedSession;

use ndarray::{Array2, Axis};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use rustfft::{num_complex::Complex, FftPlanner};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static SPEAKER_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

const SAMPLE_RATE: f32 = 16000.0;
const N_FFT: usize = 512;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;
const F_MIN: f32 = 20.0;

pub fn init_speaker(model_path: &Path) {
    SPEAKER_SESSION.get_or_init(|| {
        eprintln!("[speaker] loading wespeaker ONNX...");
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .expect("[speaker] failed to load wespeaker ONNX");
        eprintln!("[speaker] session ready");
        Mutex::new(session)
    });
}

// ── Mel filterbank helpers ────────────────────────────────────────────────────

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

fn build_mel_filterbank(n_fft: usize, n_mels: usize, sr: f32, f_min: f32) -> Array2<f32> {
    let f_max = sr / 2.0;
    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);
    let mel_points: Vec<f32> = (0..=n_mels + 1)
        .map(|i| mel_to_hz(mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32))
        .collect();

    let bin_points: Vec<usize> = mel_points
        .iter()
        .map(|&f| ((n_fft + 1) as f32 * f / sr).floor() as usize)
        .collect();

    let n_freqs = n_fft / 2 + 1;
    let mut fb = Array2::<f32>::zeros((n_mels, n_freqs));

    for m in 0..n_mels {
        let f_left = bin_points[m];
        let f_center = bin_points[m + 1];
        let f_right = bin_points[m + 2];

        for k in f_left..f_center {
            if f_center > f_left && k < n_freqs {
                fb[[m, k]] = (k - f_left) as f32 / (f_center - f_left) as f32;
            }
        }
        for k in f_center..f_right {
            if f_right > f_center && k < n_freqs {
                fb[[m, k]] = (f_right - k) as f32 / (f_right - f_center) as f32;
            }
        }
    }
    fb
}

fn hann_window(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos()))
        .collect()
}

fn compute_fbank(samples: &[f32]) -> Option<Array2<f32>> {
    let window = hann_window(N_FFT);
    let n_freqs = N_FFT / 2 + 1;
    let filterbank = build_mel_filterbank(N_FFT, N_MELS, SAMPLE_RATE, F_MIN);
    let mut planner: FftPlanner<f32> = FftPlanner::new();
    let fft = planner.plan_fft_forward(N_FFT);

    // Collect frames
    let n_frames = if samples.len() >= N_FFT {
        (samples.len() - N_FFT) / HOP_LENGTH + 1
    } else {
        return None;
    };

    if n_frames < 5 {
        eprintln!("[speaker] too few frames: {n_frames}");
        return None;
    }

    let mut fbank = Array2::<f32>::zeros((n_frames, N_MELS));

    for (fi, start) in (0..samples.len().saturating_sub(N_FFT) + 1)
        .step_by(HOP_LENGTH)
        .enumerate()
        .take(n_frames)
    {
        // Apply window
        let mut frame: Vec<Complex<f32>> = samples[start..start + N_FFT]
            .iter()
            .zip(&window)
            .map(|(&s, &w)| Complex { re: s * w, im: 0.0 })
            .collect();

        fft.process(&mut frame);

        // Power spectrum
        let power: Vec<f32> = frame[..n_freqs]
            .iter()
            .map(|c| c.re * c.re + c.im * c.im)
            .collect();

        // Apply mel filterbank
        for m in 0..N_MELS {
            let mel_energy: f32 = (0..n_freqs).map(|k| filterbank[[m, k]] * power[k]).sum();
            fbank[[fi, m]] = (mel_energy.max(1e-10_f32)).ln();
        }
    }

    // Mean-variance normalisation per mel bin
    let mean = fbank.mean_axis(Axis(0)).unwrap();
    let std = fbank.std_axis(Axis(0), 0.0);
    fbank = (fbank - &mean) / (std + 1e-8);

    eprintln!("[speaker] fbank shape: [{n_frames}, {N_MELS}]");
    Some(fbank)
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn extract_embedding(samples: &[f32]) -> Option<Vec<f32>> {
    let mutex = SPEAKER_SESSION.get()?;
    let mut session = mutex.lock().ok()?;

    eprintln!("[speaker] computing fbank for {} samples", samples.len());
    let fbank = compute_fbank(samples)?;
    let (n_frames, n_mels) = fbank.dim();

    // WeSpeaker: [batch=1, frames, mels=80]
    let input = fbank.into_shape_with_order((1, n_frames, n_mels)).ok()?;
    let tensor = Tensor::from_array(input).ok()?;

    let outputs = session.run(ort::inputs!["feats" => tensor]).ok()?;
    let raw = outputs[0].try_extract_tensor::<f32>().ok()?;
    let embedding: Vec<f32> = raw.1.iter().copied().collect();

    eprintln!("[speaker] embedding dim={}", embedding.len());
    Some(embedding)
}

// ── Speaker identification & enrollment ───────────────────────────────────────

const MATCH_THRESHOLD: f32 = 0.75;

pub async fn identify_speaker(embedding: &[f32], session: &SharedSession) {
    let pool = get_db();
    let key = get_voice_key();

    let rows = match sqlx::query_as::<_, (i64, String, String, Option<Vec<u8>>)>(
        "SELECT id, name, role, voice_embedding FROM users WHERE voice_embedding IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[speaker] db query failed: {e}");
            return;
        }
    };

    let mut best_id: Option<i64> = None;
    let mut best_name = String::new();
    let mut best_role = String::new();
    let mut best_score = 0.0_f32;

    for (id, name, role, blob) in rows {
        let Some(encrypted) = blob else { continue };
        let raw = match crypto::decrypt(key, &encrypted) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[speaker] decrypt failed for user_id={id}: {e}");
                continue;
            }
        };
        let stored: Vec<f32> = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        let score = cosine_similarity(embedding, &stored);
        eprintln!("[speaker] user_id={id} score={score:.3}");

        if score > best_score {
            best_score = score;
            best_id = Some(id);
            best_name = name;
            best_role = role;
        }
    }

    if best_score >= MATCH_THRESHOLD {
        if let Some(id) = best_id {
            session
                .lock()
                .unwrap()
                .set_user(id as i32, best_name, best_role);
        }
    } else {
        eprintln!("[speaker] no match above threshold ({best_score:.3})");
    }
}

pub async fn enroll_speaker(user_id: i32, embedding: &[f32]) -> Result<(), String> {
    let pool = get_db();
    let key = get_voice_key();
    let raw: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
    let encrypted = crypto::encrypt(key, &raw)?;
    sqlx::query("UPDATE users SET voice_embedding = ? WHERE id = ?")
        .bind(encrypted)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("[speaker] enroll failed: {e}"))?;
    eprintln!("[speaker] enrolled encrypted embedding for user_id={user_id}");
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}
