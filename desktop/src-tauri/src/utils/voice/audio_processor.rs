const SILENCE_THRESHOLD: f32 = 0.01;
const PRE_EMPHASIS: f32 = 0.97;
const MIN_RMS: f32 = 0.005;

pub fn process_f32(samples: Vec<f32>) -> Vec<f32> {
    if samples.is_empty() {
        return vec![];
    }

    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    eprintln!("[audio] rms={rms:.4}");

    if rms < MIN_RMS {
        eprintln!("[audio] rejected — below rms threshold (rms={rms:.4})");
        return vec![];
    }

    // Pre-emphasis
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
        .unwrap_or(emphasized.len());

    if start >= end {
        eprintln!("[audio] rejected — nothing above threshold after trim");
        return vec![];
    }

    let trimmed = &emphasized[start..end];

    // Reject if the active (above-threshold) portion is less than 20% of the chunk —
    // real speech is consistently above threshold, noise spikes briefly then drops
    let active_samples = trimmed
        .iter()
        .filter(|s| s.abs() > SILENCE_THRESHOLD)
        .count();
    let active_ratio = active_samples as f32 / trimmed.len() as f32;
    if active_ratio < 0.08 {
        eprintln!("[audio] rejected — sparse activity (active_ratio={active_ratio:.2})");
        return vec![];
    }

    // Normalize
    let max = trimmed.iter().map(|s| s.abs()).fold(0_f32, f32::max);
    if max == 0.0 {
        return vec![];
    }

    trimmed.iter().map(|s| s / max).collect()
}
