use serde::Serialize;
use std::cmp::Reverse;

#[derive(Debug, Serialize, Clone)]
pub struct ResolvedCommand {
    pub intent: String,
    pub args: Vec<String>,
    pub raw: String,
}

/// (intent, triggers[])
/// Triggers are matched anywhere in the utterance, longest match wins.
const INTENT_MAP: &[(&str, &[&str])] = &[
    // UI
    ("open", &["open", "show", "launch", "go to", "start"]),
    ("close", &["close", "hide", "dismiss", "exit", "quit"]),
    ("play", &["play", "watch"]),
    ("stop", &["stop", "pause", "freeze"]),
    (
        "search",
        &["search for", "search", "find", "look up", "look for"],
    ),
    ("volume", &["volume", "louder", "quieter", "mute", "unmute"]),
    ("help", &["help me", "help", "how do", "what can"]),
    // auth
    (
        "login",
        &["log in", "login", "sign in", "log me in", "sign me in"],
    ),
    (
        "signup",
        &[
            "sign up",
            "signup",
            "register",
            "create account",
            "create an account",
            "new account",
        ],
    ),
    // answers
    (
        "submit_answer",
        &[
            "my answer is",
            "i think the answer",
            "i think it",
            "i say",
            "answer is",
            "submit answer",
            "submit my answer",
        ],
    ),
    (
        "my_answers",
        &[
            "my answers",
            "my results",
            "show my answers",
            "show answers",
        ],
    ),
    // kids
    (
        "my_tags",
        &[
            "my interests",
            "my tags",
            "list my tags",
            "show my tags",
            "what are my tags",
            "what are my interests",
        ],
    ),
    (
        "add_tag",
        &["add interest", "add tag", "i like", "i enjoy", "i love"],
    ),
    (
        "my_videos",
        &[
            "my videos",
            "my assignments",
            "assigned videos",
            "show my videos",
            "list my videos",
        ],
    ),
    (
        "assign_video",
        &["assign video", "assign this video", "add video"],
    ),
    (
        "recommendations",
        &[
            "recommend",
            "recommendations",
            "suggest",
            "what should i watch",
            "what can i watch",
            "suggest something",
        ],
    ),
    // parents
    (
        "my_kids",
        &[
            "my kids",
            "my children",
            "list kids",
            "show kids",
            "my students",
        ],
    ),
    // videos
    (
        "download_video",
        &["download video", "download this", "download"],
    ),
    (
        "video_tags",
        &["video tags", "tags for this", "tags for video", "what tags"],
    ),
    // questions
    (
        "get_questions",
        &[
            "quiz me",
            "test me",
            "ask me",
            "give me questions",
            "show questions",
            "questions for",
        ],
    ),
    (
        "generate_questions",
        &[
            "generate questions",
            "make questions",
            "create quiz",
            "create questions",
        ],
    ),
    // tags
    (
        "all_tags",
        &["all tags", "list all tags", "show all tags", "list tags"],
    ),
    (
        "create_tag",
        &["create tag", "new tag", "add new tag", "make a tag"],
    ),
    // frames
    (
        "extract_frames",
        &["extract frames", "get frames", "grab frames"],
    ),
];

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
    let lower = sanitize(&raw.to_lowercase());

    let tokens: Vec<String> = lower.split_whitespace().map(str::to_string).collect();

    if tokens.is_empty() {
        return ResolvedCommand {
            intent: "wake_only".to_string(),
            args: vec![],
            raw,
        };
    }

    // Collect all matches, pick the longest trigger (most specific wins)
    let mut matches: Vec<(&str, usize, usize)> = Vec::new(); // (intent, trigger_len, pos)
    for (intent, triggers) in INTENT_MAP {
        for trigger in *triggers {
            if let Some(pos) = lower.find(trigger) {
                matches.push((intent, trigger.len(), pos));
            }
        }
    }

    if !matches.is_empty() {
        matches.sort_by_key(|(_, len, _)| Reverse(*len));
        let (intent, trigger_len, pos) = matches[0];

        // args = everything after the matched trigger
        let after = sanitize(lower[pos + trigger_len..].trim());
        let args: Vec<String> = after
            .split_whitespace()
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect();

        eprintln!("[resolver] intent={intent} args={args:?}");
        return ResolvedCommand {
            intent: intent.to_string(),
            args,
            raw,
        };
    }

    // Nothing matched — treat as chat
    eprintln!("[resolver] intent=chat args={tokens:?}");
    ResolvedCommand {
        intent: "chat".to_string(),
        args: tokens,
        raw,
    }
}
