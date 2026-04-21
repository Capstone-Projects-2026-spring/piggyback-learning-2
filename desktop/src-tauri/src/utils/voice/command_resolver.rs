use super::intent_classifier;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ResolvedCommand {
    pub intent: String,
    pub args: Vec<String>,
    pub raw: String,
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

pub fn resolve(command_text: &str) -> ResolvedCommand {
    let raw = command_text.trim().to_string();
    let clean = sanitize(&raw.to_lowercase());
    let tokens: Vec<String> = clean.split_whitespace().map(str::to_string).collect();

    if tokens.is_empty() {
        return ResolvedCommand {
            intent: "wake_only".to_string(),
            args: vec![],
            raw,
        };
    }

    let intent = intent_classifier::classify(&clean);

    eprintln!("[resolver] '{raw}' → intent={intent}");
    ResolvedCommand {
        intent,
        args: tokens,
        raw,
    }
}
