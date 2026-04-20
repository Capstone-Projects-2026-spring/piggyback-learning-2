use serde::Serialize;

const WAKE_PHRASES: &[&str] = &[
    // canonical
    "hey peppa",
    "hey pepper",
    "hi peppa",
    "hi pepper",
    "hello peppa",
    "hello pepper",
    // without hey/hi
    "peppa",
    "pepper",
    // mishearings
    "hey papa",
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

#[derive(Debug, Serialize)]
pub struct WakeWordResult {
    pub wake_detected: bool,
    pub command_text: String,
}

/// Strip punctuation and normalize whitespace
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

    // Try longest phrases first so "hey peppa do x" doesn't match just "peppa"
    // and leave "hey " as part of command_text
    let mut phrases_sorted = WAKE_PHRASES.to_vec();
    phrases_sorted.sort_by_key(|p| std::cmp::Reverse(p.len()));

    for phrase in &phrases_sorted {
        if let Some(pos) = normalized.find(phrase) {
            let after = sanitize(normalized[pos + phrase.len()..].trim());
            eprintln!("[wake] matched '{phrase}' → command='{after}'");
            return WakeWordResult {
                wake_detected: true,
                command_text: after,
            };
        }
    }

    // Secondary check — fuzzy: does the transcript contain any wake-word token?
    // Catches cases like "okay peppa" or "yeah pepper stop"
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    const WAKE_TOKENS: &[&str] = &["peppa", "pepper", "pepa", "peper", "hepa", "paper", "papa"];
    for (i, token) in tokens.iter().enumerate() {
        if WAKE_TOKENS.contains(token) {
            // everything after this token is the command
            let after = sanitize(&tokens[i + 1..].join(" "));
            eprintln!("[wake] fuzzy match on token '{token}' → command='{after}'");
            return WakeWordResult {
                wake_detected: true,
                command_text: after,
            };
        }
    }

    eprintln!("[wake] no wake word in: '{normalized}'");
    WakeWordResult {
        wake_detected: false,
        command_text: String::new(),
    }
}
