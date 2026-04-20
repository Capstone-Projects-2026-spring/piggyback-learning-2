use serde::Serialize;

use crate::utils::voice::{
    audio_processor,
    command_resolver::{self, ResolvedCommand},
    stt, wake_word,
};

#[derive(Debug, Serialize)]
pub struct VoiceResult {
    /// Full transcript from Vosk
    pub transcript: String,
    /// Whether "hey peppa" was in the transcript
    pub wake_detected: bool,
    /// Resolved command — None if wake word was not detected
    pub command: Option<ResolvedCommand>,
}

/// Receives raw WAV bytes from the frontend (captured by MediaRecorder),
/// runs the full pipeline: parse → clean → STT → wake word → resolve.
#[tauri::command]
pub fn process_audio(wav_bytes: Vec<u8>) -> Result<VoiceResult, String> {
    // 1. Parse and clean the WAV
    let audio = audio_processor::parse_wav(&wav_bytes)?;

    // 2. Transcribe with Vosk
    let transcript = stt::transcribe(&audio.samples)?;

    if transcript.is_empty() {
        return Ok(VoiceResult {
            transcript,
            wake_detected: false,
            command: None,
        });
    }

    // 3. Wake word detection
    let wake = wake_word::detect(&transcript);

    if !wake.wake_detected {
        return Ok(VoiceResult {
            transcript,
            wake_detected: false,
            command: None,
        });
    }

    // 4. Resolve intent from whatever came after "hey peppa"
    let command = command_resolver::resolve(&wake.command_text);

    Ok(VoiceResult {
        transcript,
        wake_detected: true,
        command: Some(command),
    })
}
