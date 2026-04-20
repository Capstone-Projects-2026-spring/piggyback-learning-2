pub fn rms_energy(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum: f32 = samples.iter().map(|&x| (x as f32).powi(2)).sum();

    (sum / samples.len() as f32).sqrt()
}

pub fn detect_mood(samples: &[i16]) -> (String, f32) {
    let energy = rms_energy(samples);

    // Using ONLY energy thresholds now
    let mood = if energy < 2500.0 {
        "bored"
    } else if energy > 4500.0 {
        "excited"
    } else {
        "neutral"
    };

    (mood.to_string(), energy)
}