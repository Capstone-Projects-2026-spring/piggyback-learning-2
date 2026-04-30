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

/// Process a mono 16kHz chunk for STT input: peak normalize only.
/// Returns empty vec if the chunk is silent.
pub fn process_f32(samples: Vec<f32>) -> Vec<f32> {
    if samples.is_empty() {
        return vec![];
    }

    let max = samples.iter().map(|s| s.abs()).fold(0_f32, f32::max);
    if max < 1e-6 {
        eprintln!("[audio] rejected - silent");
        return vec![];
    }

    samples.iter().map(|s| s / max).collect()
}
