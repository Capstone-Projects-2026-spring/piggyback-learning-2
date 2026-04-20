use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ResolvedCommand {
    pub intent: String,
    pub args: Vec<String>,
    pub raw: String,
}

/// Maps first-token triggers to intents.
/// Order matters — first match wins.
const INTENT_MAP: &[(&str, &[&str])] = &[
    ("open", &["open", "show", "launch", "go", "start"]),
    ("close", &["close", "hide", "dismiss", "exit", "quit"]),
    ("play", &["play", "watch"]),
    ("stop", &["stop", "pause", "freeze"]),
    ("search", &["search", "find", "look"]),
    ("volume", &["volume", "louder", "quieter", "mute", "unmute"]),
    ("help", &["help", "what", "how"]),
];

pub fn resolve(command_text: &str) -> ResolvedCommand {
    let raw = command_text.trim().to_string();
    let tokens: Vec<String> = raw
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect();

    if tokens.is_empty() {
        return ResolvedCommand {
            intent: "wake_only".to_string(), // wake word with no command
            args: vec![],
            raw,
        };
    }

    let first = tokens[0].as_str();
    for (intent, triggers) in INTENT_MAP {
        if triggers.contains(&first) {
            return ResolvedCommand {
                intent: intent.to_string(),
                args: tokens[1..].to_vec(),
                raw,
            };
        }
    }

    // Nothing matched — treat whole utterance as a chat/query
    ResolvedCommand {
        intent: "chat".to_string(),
        args: tokens,
        raw,
    }
}
