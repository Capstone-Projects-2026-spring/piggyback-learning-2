use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::{Mutex, OnceLock};

static EMBEDDER: OnceLock<Mutex<TextEmbedding>> = OnceLock::new();

const INTENT_EXAMPLES: &[(&str, &[&str])] = &[
    (
        "open",
        &["open", "show me", "launch", "go to", "start", "bring up"],
    ),
    (
        "close",
        &["close", "hide", "dismiss", "exit", "quit", "shut down"],
    ),
    (
        "play",
        &["play", "watch", "play video", "start video", "watch this"],
    ),
    (
        "stop",
        &["stop", "pause", "freeze", "stop playing", "pause the video"],
    ),
    (
        "search",
        &[
            "search for spiderman",
            "search for dinosaur videos",
            "find videos about space",
            "look up minecraft videos",
            "find me videos about robots",
            "show me videos about animals",
            "search youtube for cooking",
            "find spiderman on youtube",
            "look for science videos",
            "search for spider-man",
            "find videos of football",
            "i want to watch something about",
            "can you find videos about",
        ],
    ),
    (
        "volume",
        &[
            "volume up",
            "volume down",
            "louder",
            "quieter",
            "mute",
            "unmute",
        ],
    ),
    (
        "help",
        &[
            "help",
            "help me",
            "what can you do",
            "how do I use this",
            "what commands are available",
        ],
    ),
    (
        "add_kid",
        &[
            "let's create my kids account",
            "add a kid",
            "create a kid account",
            "add a child",
            "set up a kid",
            "new kid",
            "register a child",
            "enroll a kid",
            "add my son",
            "add my daughter",
            "create account for my kid",
            "let's create my kid's account",
            "i want to add a child",
            "sign up my kid",
            "make an account for my child",
        ],
    ),
    (
        "submit_answer",
        &[
            "my answer is",
            "I think the answer is",
            "I say",
            "the answer is",
            "submit my answer",
            "I think it's",
            "my response is",
        ],
    ),
    (
        "my_answers",
        &[
            "my answers",
            "my results",
            "show my answers",
            "how did I do",
            "my quiz results",
            "show my results",
        ],
    ),
    (
        "add_tag",
        &[
            "add interest",
            "add tag",
            "I enjoy football",
            "I love dinosaurs",
            "I like space",
            "add this to my interests",
            "I'm interested in cooking",
            "my kid likes dinosaurs",
            "my daughter likes reading",
            "my son likes robots",
            "he likes football",
            "she loves painting",
            "they like math",
            "is really into space",
            "is interested in animals",
            "enjoys reading",
            "loves cooking",
            "mark me as interested in",
            "remember that I like",
            "note that she likes",
        ],
    ),
    (
        "my_tags",
        &[
            "what are my interests",
            "show my tags",
            "list my tags",
            "what tags do I have",
            "show my interests",
            "list my interests",
            "what interests have been saved",
            "what do you know I like",
            "show what I like",
            "what are my saved interests",
        ],
    ),
    (
        "my_videos",
        &[
            "my videos",
            "my assignments",
            "assigned videos",
            "show my assigned videos",
            "what videos do I have",
            "list my videos",
            "what have I been assigned",
            "videos assigned to me",
        ],
    ),
    (
        "assign_video",
        &[
            "assign video",
            "assign this video",
            "add this video",
            "add video for kid",
            "give this video to",
        ],
    ),
    (
        "recommendations",
        &[
            "recommend something",
            "suggest a video",
            "what should I watch",
            "what can I watch",
            "give me recommendations",
            "suggest something",
            "what's good to watch",
            "find me something to watch",
        ],
    ),
    (
        "my_kids",
        &[
            "my kids",
            "my children",
            "list kids",
            "show my kids",
            "who are my students",
            "show my children",
        ],
    ),
    (
        "download_video",
        &[
            "download video",
            "download this",
            "download",
            "save this video",
            "save video locally",
        ],
    ),
    (
        "get_questions",
        &[
            "quiz me",
            "test me",
            "ask me questions",
            "give me a quiz",
            "questions for this video",
            "start quiz",
            "I want to be quizzed",
        ],
    ),
    (
        "generate_questions",
        &[
            "generate questions",
            "make questions",
            "create a quiz",
            "create questions for this",
            "make a quiz",
        ],
    ),
    (
        "all_tags",
        &[
            "all tags",
            "list all tags",
            "show all tags",
            "what tags exist",
            "list tags",
        ],
    ),
    (
        "create_tag",
        &[
            "create tag",
            "new tag",
            "add new tag",
            "make a tag",
            "create a new tag",
        ],
    ),
    (
        "extract_frames",
        &[
            "extract frames",
            "get frames",
            "grab frames from video",
            "pull frames",
        ],
    ),
    (
        "chat",
        &[
            "what is",
            "tell me about",
            "explain",
            "who is",
            "why does",
            "how does",
            "what does",
            "can you explain",
            "I want to know about",
        ],
    ),
];

/// Below this cosine similarity we fall back to chat
const MATCH_THRESHOLD: f32 = 0.42;

pub fn init_classifier() {
    std::thread::spawn(|| {
        eprintln!("[classifier] loading fastembed model...");
        match TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        ) {
            Ok(model) => {
                EMBEDDER.set(Mutex::new(model)).ok();
                eprintln!("[classifier] fastembed model ready");
            }
            Err(e) => eprintln!("[classifier] failed to load: {e}"),
        }
    });
}

pub fn classify(transcript: &str) -> String {
    let Some(mutex) = EMBEDDER.get() else {
        eprintln!("[classifier] model not ready — keyword fallback");
        return keyword_fallback(transcript);
    };

    let mut embedder = mutex.lock().unwrap();

    let transcript_embs = match embedder.embed(vec![transcript], None) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[classifier] embed failed: {e}");
            return keyword_fallback(transcript);
        }
    };
    let t_emb = &transcript_embs[0];

    let mut best_intent = "chat".to_string();
    let mut best_score = 0.0_f32;

    for (intent, examples) in INTENT_EXAMPLES {
        let example_vec: Vec<&str> = examples.to_vec();
        let Ok(embs) = embedder.embed(example_vec, None) else {
            continue;
        };

        let score = embs
            .iter()
            .map(|e| cosine_similarity(t_emb, e))
            .fold(0.0_f32, f32::max);

        eprintln!("[classifier] {intent:20} score={score:.3}");

        if score > best_score {
            best_score = score;
            best_intent = intent.to_string();
        }
    }

    if best_score < MATCH_THRESHOLD {
        eprintln!("[classifier] score {best_score:.3} below threshold → chat");
        return "chat".to_string();
    }

    eprintln!("[classifier] '{transcript}' → {best_intent} ({best_score:.3})");
    best_intent
}

/// Public helper so other modules can embed arbitrary strings
/// using the already-loaded fastembed model.
pub fn embed_strings(inputs: &[&str]) -> Result<Vec<Vec<f32>>, String> {
    let Some(mutex) = EMBEDDER.get() else {
        return Err("[classifier] embedder not ready".to_string());
    };
    let mut embedder = mutex.lock().unwrap();
    embedder
        .embed(inputs.to_vec(), None)
        .map_err(|e| format!("[classifier] embed failed: {e}"))
}

fn keyword_fallback(text: &str) -> String {
    let l = text.to_lowercase();
    if l.contains("my video") || l.contains("assigned") {
        return "my_videos".to_string();
    }
    // Retrieval: must be a question or list request
    if (l.contains("my tag") || l.contains("my interest"))
        && (l.contains("show") || l.contains("list") || l.contains("what"))
    {
        return "my_tags".to_string();
    }
    // Assignment: statements of preference
    if l.contains("likes ")
        || l.contains("loves ")
        || l.contains("enjoys ")
        || l.contains("i like ")
        || l.contains("i love ")
        || l.contains("i enjoy ")
        || l.contains("interested in")
    {
        return "add_tag".to_string();
    }
    if l.contains("recommend") || l.contains("suggest") {
        return "recommendations".to_string();
    }
    if l.contains("quiz") || l.contains("test me") {
        return "get_questions".to_string();
    }
    if l.contains("download") {
        return "download_video".to_string();
    }
    if l.contains("answer") {
        return "submit_answer".to_string();
    }
    if l.contains("my kid") || l.contains("children") {
        return "my_kids".to_string();
    }
    if l.contains("play") {
        return "play".to_string();
    }
    if l.contains("stop") || l.contains("pause") {
        return "stop".to_string();
    }
    if l.contains("help") {
        return "help".to_string();
    }
    "chat".to_string()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}
