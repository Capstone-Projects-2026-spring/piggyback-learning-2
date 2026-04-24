use serde::Serialize;

const WAKE_PHRASES: &[&str] = &[
    "hey peppa",
    "hey pepper",
    "hi peppa",
    "hi pepper",
    "hello peppa",
    "hello pepper",
    "hello people",
    "peppa",
    "pepper",
    "people",
    "hey papa",
    "hey people",
    "hey pappa",
    "a peppa",
    "a pepper",
    "hey paper",
    "hi paper",
    "hello paper",
    "paper",
    "papa",
    "peper",
    "pepa",
    "hepa",
];

const WAKE_TOKENS: &[&str] = &[
    "peppa", "pepper", "pepa", "peper", "hepa", "paper", "papa", "people",
];

#[derive(Debug, Serialize)]
pub struct WakeWordResult {
    pub wake_detected: bool,
}

fn sanitize(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn detect(transcript: &str) -> WakeWordResult {
    let normalized = sanitize(&transcript.trim().to_lowercase());

    let mut phrases_sorted = WAKE_PHRASES.to_vec();
    phrases_sorted.sort_by_key(|p| std::cmp::Reverse(p.len()));

    for phrase in &phrases_sorted {
        if normalized.contains(phrase) {
            eprintln!("[wake] matched phrase '{phrase}' in '{normalized}'");
            return WakeWordResult {
                wake_detected: true,
            };
        }
    }

    // Fuzzy token fallback
    for token in normalized.split_whitespace() {
        if WAKE_TOKENS.contains(&token) {
            eprintln!("[wake] fuzzy token match '{token}' in '{normalized}'");
            return WakeWordResult {
                wake_detected: true,
            };
        }
    }

    eprintln!("[wake] no wake word in: '{normalized}'");
    WakeWordResult {
        wake_detected: false,
    }
}
