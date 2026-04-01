use vosk::{CompleteResult, Recognizer};

use crate::utils::voice::state::VOSK_MODEL;

pub fn transcribe(samples: &[i16]) -> Result<String, String> {
    if samples.is_empty() {
        return Ok(String::new());
    }

    let mut recognizer = Recognizer::new(&VOSK_MODEL, 16000.0)
        .ok_or_else(|| "Failed to create recognizer".to_string())?;

    recognizer
        .accept_waveform(samples)
        .map_err(|e| format!("Waveform error: {:?}", e))?;

    let result = recognizer.final_result();

    let text = match result {
        CompleteResult::Single(res) => res.text.to_string(),
        CompleteResult::Multiple(res) => res
            .alternatives
            .first()
            .map(|a| a.text.to_string())
            .unwrap_or_default(),
    };

    Ok(text)
}
