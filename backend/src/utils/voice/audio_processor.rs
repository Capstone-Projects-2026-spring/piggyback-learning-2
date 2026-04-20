use hound;

pub struct AudioData {
    pub samples: Vec<i16>,     // Normalized, use this for STT (Vosk)
    pub raw_samples: Vec<i16>, // Un-normalized, use this for Mood detection
    pub sample_rate: u32,
}

pub fn parse_wav(bytes: &[u8]) -> Result<AudioData, String> {
    let reader = hound::WavReader::new(std::io::Cursor::new(bytes));
    if reader.is_err() {
        println!("{:#?}", reader.err());
        return Err("Unknown error occurred with the WavReader".to_string());
    }
    let reader = reader.unwrap();
    let spec = reader.spec();

    if spec.channels != 1 {
        return Err("Audio must be mono".to_string());
    }

    if spec.sample_rate != 16000 {
        return Err("Sample rate must be 16kHz".to_string());
    }

    let initial_samples: Vec<i16> = reader
        .into_samples::<i16>()
        .filter_map(Result::ok)
        .collect();

    // 1. Trim dead silence from the beginning and end
    // LOWERED THRESHOLD: 200 cuts out dead static, but keeps quiet whispering!
    let trimmed_samples = trim_silence(&initial_samples, 200);
    
    // 2. Capture the pure audio for Mood Detection BEFORE the noise gate!
    let raw_for_mood = trimmed_samples.clone();
    
    // 3. Apply noise gate and normalize for Vosk STT ONLY
    let mut cleaned_samples = noise_gate(&trimmed_samples, 600);
    normalize(&mut cleaned_samples);

    Ok(AudioData {
        samples: cleaned_samples,
        raw_samples: raw_for_mood,
        sample_rate: spec.sample_rate,
    })
}

// --- FIXED HELPER FUNCTIONS ---

fn trim_silence(samples: &[i16], threshold: i16) -> Vec<i16> {
    // Cast to i32 before abs() to prevent i16::MIN overflow panics
    let start = samples.iter().position(|&x| (x as i32).abs() > threshold as i32);
    let end = samples.iter().rposition(|&x| (x as i32).abs() > threshold as i32);

    match (start, end) {
        // THE FIX: Use ..= to make the slice inclusive of the final sample!
        (Some(s), Some(e)) => samples[s..=e].to_vec(),
        
        // THE FIX: If the entire clip is below the threshold, don't delete it!
        // Return it as-is so Vosk can still try to transcribe the whispering.
        _ => samples.to_vec(), 
    }
}

fn noise_gate(samples: &[i16], threshold: i16) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| if (s as i32).abs() < threshold as i32 { 0 } else { s })
        .collect()
}

fn normalize(samples: &mut [i16]) {
    let max = samples.iter().map(|&x| (x as i32).abs()).max().unwrap_or(1) as f32;

    if max == 0.0 {
        return;
    }

    let scale = i16::MAX as f32 / max;

    for s in samples.iter_mut() {
        *s = (*s as f32 * scale) as i16;
    }
}