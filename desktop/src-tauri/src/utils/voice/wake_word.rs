use serde::Serialize;

/// Phonetic variants Vosk might produce for "hey peppa"
const WAKE_PHRASES: &[&str] = &[
    "hey peppa",
    "hey pepper",
    "a peppa",
    "peppa",
    "hey pepa",
    "a pepper",
];

#[derive(Debug, Serialize)]
pub struct WakeWordResult {
    /// Whether the wake word was detected in this transcript
    pub wake_detected: bool,
    /// Everything after the wake word — empty string if wake word only
    pub command_text: String,
}

/// Check a transcript for the wake word and strip it out.
/// Returns the command text that follows (may be empty).
pub fn detect(transcript: &str) -> WakeWordResult {
    let normalized = transcript.trim().to_lowercase();

    for phrase in WAKE_PHRASES {
        if let Some(pos) = normalized.find(phrase) {
            let after = normalized[pos + phrase.len()..].trim().to_string();
            return WakeWordResult {
                wake_detected: true,
                command_text: after,
            };
        }
    }

    WakeWordResult {
        wake_detected: false,
        command_text: String::new(),
    }
}
