const SILENCE_THRESHOLD: f32 = 0.01;
const PRE_EMPHASIS: f32 = 0.97;
const MIN_RMS: f32 = 0.005;
/// Minimum fraction of samples above threshold to count as real speech.
/// Noise spikes briefly then drops,  real speech stays consistently above.
const MIN_ACTIVE_RATIO: f32 = 0.08;

/// Convert multi-channel interleaved samples to mono by averaging channels.
pub fn to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    data.chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Resample from `from` Hz to `to` Hz using linear interpolation.
/// Returns the input unchanged if rates are equal.
pub fn resample(samples: Vec<f32>, from: u32, to: u32) -> Vec<f32> {
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

/// Run the full processing pipeline on a mono 16kHz chunk:
/// RMS gate -> pre-emphasis -> silence trim -> activity check -> normalize.
/// Returns an empty vec if the chunk is rejected at any stage.
pub fn process_f32(samples: Vec<f32>) -> Vec<f32> {
    if samples.is_empty() {
        return vec![];
    }

    // RMS gate — reject background noise before doing any work
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    if rms < MIN_RMS {
        eprintln!("[audio] rejected — rms={rms:.4} below threshold");
        return vec![];
    }
    eprintln!("[audio] rms={rms:.4}");

    // Pre-emphasis — boost high frequencies for cleaner Whisper input
    let mut emphasized = Vec::with_capacity(samples.len());
    emphasized.push(samples[0]);
    for i in 1..samples.len() {
        emphasized.push(samples[i] - PRE_EMPHASIS * samples[i - 1]);
    }

    // Trim leading/trailing silence
    let start = emphasized
        .iter()
        .position(|s| s.abs() > SILENCE_THRESHOLD)
        .unwrap_or(0);
    let end = emphasized
        .iter()
        .rposition(|s| s.abs() > SILENCE_THRESHOLD)
        .unwrap_or(0);

    if start >= end {
        eprintln!("[audio] rejected — nothing above silence threshold after trim");
        return vec![];
    }

    let trimmed = &emphasized[start..end];

    // Activity ratio check — filters out single noise spikes
    let active_ratio = trimmed
        .iter()
        .filter(|s| s.abs() > SILENCE_THRESHOLD)
        .count() as f32
        / trimmed.len() as f32;

    if active_ratio < MIN_ACTIVE_RATIO {
        eprintln!("[audio] rejected — sparse activity (ratio={active_ratio:.2})");
        return vec![];
    }

    // Peak normalize
    let max = trimmed.iter().map(|s| s.abs()).fold(0_f32, f32::max);
    if max == 0.0 {
        return vec![];
    }

    trimmed.iter().map(|s| s / max).collect()
}
