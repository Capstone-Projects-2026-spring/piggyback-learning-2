pub fn process_f32(samples: Vec<f32>) -> Vec<f32> {
    // Trim silence
    let threshold = 0.01_f32;
    let start = samples
        .iter()
        .position(|s| s.abs() > threshold)
        .unwrap_or(0);
    let end = samples
        .iter()
        .rposition(|s| s.abs() > threshold)
        .unwrap_or(samples.len());
    if start >= end {
        return vec![];
    }
    let trimmed = &samples[start..end];

    // Normalize
    let max = trimmed.iter().map(|s| s.abs()).fold(0_f32, f32::max);
    if max == 0.0 {
        return vec![];
    }
    trimmed.iter().map(|s| s / max).collect()
}
