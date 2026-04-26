use crate::utils::text::normalize;
use serde::Serialize;

/// All phrases that should count as a wake event.
/// Sorted longest-first at runtime so "hey jarvis" matches before "jarvis" alone.
/// Includes common Whisper mishearings of "Jarvis" across accents.
const WAKE_PHRASES: &[&str] = &[
    "hey jarvis",
    "hi jarvis",
    "hello jarvis",
    "okay jarvis",
    "ok jarvis",
    "jarvis",
];

/// Single-token fallback for when Whisper drops the greeting word entirely.
const WAKE_TOKENS: &[&str] = &["jarvis", "jarvas", "jarves", "jarvi", "jarviz"];

#[derive(Debug, Serialize)]
pub struct WakeWordResult {
    pub wake_detected: bool,
}

pub fn detect(transcript: &str) -> WakeWordResult {
    let normalized = normalize(transcript.trim());

    let mut phrases_sorted = WAKE_PHRASES.to_vec();
    phrases_sorted.sort_by_key(|p| std::cmp::Reverse(p.len()));

    for phrase in &phrases_sorted {
        if normalized.contains(phrase) {
            eprintln!("[wake] phrase match '{phrase}' in '{normalized}'");
            return WakeWordResult {
                wake_detected: true,
            };
        }
    }

    for token in normalized.split_whitespace() {
        if WAKE_TOKENS.contains(&token) {
            eprintln!("[wake] token match '{token}' in '{normalized}'");
            return WakeWordResult {
                wake_detected: true,
            };
        }
    }

    eprintln!("[wake] no match in '{normalized}'");
    WakeWordResult {
        wake_detected: false,
    }
}
