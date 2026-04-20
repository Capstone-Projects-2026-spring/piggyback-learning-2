use crate::utils::voice::state::get_model;
use vosk::{CompleteResult, Recognizer};

pub fn transcribe(samples: &[i16]) -> Result<String, String> {
    if samples.is_empty() {
        return Ok(String::new());
    }

    let mut recognizer = Recognizer::new(get_model(), 16000.0)
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
