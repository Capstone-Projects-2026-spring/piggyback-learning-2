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

    // 1. Trim silence and noise gate FIRST
    let mut cleaned_samples = trim_silence(&initial_samples, 500);
    cleaned_samples = noise_gate(&cleaned_samples, 600);
    
    // 2. Clone the cleaned (but NOT normalized) audio for Mood Detection
    let raw_for_mood = cleaned_samples.clone();
    
    // 3. Normalize the audio for Vosk STT
    normalize(&mut cleaned_samples);

    Ok(AudioData {
        samples: cleaned_samples, // Loud version
        raw_samples: raw_for_mood, // Quiet/Accurate version
        sample_rate: spec.sample_rate,
    })
}

// NOTE: I kept your helper functions exactly the same!
fn trim_silence(samples: &[i16], threshold: i16) -> Vec<i16> {
    let start = samples
        .iter()
        .position(|x| x.abs() > threshold)
        .unwrap_or(0);

    let end = samples
        .iter()
        .rposition(|x| x.abs() > threshold)
        .unwrap_or(samples.len());

    if start >= end {
        return vec![];
    }

    samples[start..end].to_vec()
}

fn noise_gate(samples: &[i16], threshold: i16) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| if s.abs() < threshold { 0 } else { s })
        .collect()
}

fn normalize(samples: &mut [i16]) {
    let max = samples.iter().map(|x| x.abs()).max().unwrap_or(1) as f32;

    if max == 0.0 {
        return;
    }

    let scale = i16::MAX as f32 / max;

    for s in samples {
        *s = (*s as f32 * scale) as i16;
    }
}