use super::intent_classifier;
use crate::utils::text::normalize;
use crate::utils::voice::intent::Intent;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ResolvedCommand {
    pub intent: Intent,
    pub args: Vec<String>,
    pub raw: String,
}

pub fn resolve(transcript: &str) -> ResolvedCommand {
    let raw = transcript.trim().to_string();
    let clean = normalize(&raw);

    if clean.is_empty() {
        return ResolvedCommand {
            intent: Intent::WakeOnly,
            args: vec![],
            raw,
        };
    }

    let tokens: Vec<String> = clean.split_whitespace().map(str::to_string).collect();
    let intent = intent_classifier::classify(&clean);

    eprintln!("[resolver] '{raw}' - {intent:?}");
    ResolvedCommand {
        intent,
        args: tokens,
        raw,
    }
}
