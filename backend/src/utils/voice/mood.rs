pub fn rms_energy(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum: f32 = samples.iter().map(|&x| (x as f32).powi(2)).sum();

    (sum / samples.len() as f32).sqrt()
}

pub fn zero_crossing_rate(samples: &[i16]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }

    let mut crossings = 0;

    for i in 1..samples.len() {
        if (samples[i - 1] >= 0 && samples[i] < 0) || (samples[i - 1] < 0 && samples[i] >= 0) {
            crossings += 1;
        }
    }

    crossings as f32 / samples.len() as f32
}

pub fn detect_mood(samples: &[i16]) -> (String, f32) {
    let energy = rms_energy(samples);
    let zcr = zero_crossing_rate(samples);

    // Adjusted thresholds now that we are using raw un-normalized audio
    let mood = if energy < 800.0 && zcr < 0.06 {
        // Very quiet volume, low frequency variation (monotone)
        "bored"
    } else if energy > 1800.0 && zcr > 0.12 {
        // High volume, high frequency variation (sharp sounds, yelling/laughing)
        "excited"
    } else {
        "neutral"
    };

    (mood.to_string(), energy)
}