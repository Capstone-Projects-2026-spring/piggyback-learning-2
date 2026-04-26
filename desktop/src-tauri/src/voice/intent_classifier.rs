use crate::utils::text::cosine_similarity;
use crate::voice::intent::Intent;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::{Mutex, OnceLock};

const MATCH_THRESHOLD: f32 = 0.45;

const INTENT_EXAMPLES: &[(&str, &[&str])] = &[
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
            "find videos of football",
            "i want to watch something about",
            "can you find videos about",
            "search for",
            "find videos of",
            "look up videos of",
        ],
    ),
    (
        "add_kid",
        &[
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
            "i want to add a child",
            "sign up my kid",
            "make an account for my child",
        ],
    ),
    (
        "kid_results",
        &[
            "show me adam's results",
            "how did sarah do",
            "show results for jake",
            "what were emma's answers",
            "show me the results for tom",
            "how did lily do on the video",
            "check jake's progress",
            "show adam's quiz results",
            "how well did noah do",
            "what did emma score",
            "show me how jake did",
            "pull up sarah's results",
            "how is adam doing",
            "results for my kid",
            "show kid results",
        ],
    ),
    (
        "add_tag",
        &[
            "i enjoy football",
            "i love dinosaurs",
            "i like space",
            "i'm interested in cooking",
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
            "remember that i like",
            "note that she likes",
        ],
    ),
    (
        "my_videos",
        &[
            "my videos",
            "my assignments",
            "assigned videos",
            "show my assigned videos",
            "what videos do i have",
            "list my videos",
            "what have i been assigned",
            "videos assigned to me",
        ],
    ),
    (
        "assign_video",
        &[
            "assign it to emma",
            "assign this to jake",
            "give it to emma",
            "give this to jake",
            "assign to emma",
            "this one is for jake",
            "assign this video to emma",
            "give this video to jake",
        ],
    ),
    (
        "watch_video",
        &[
            "watch this",
            "watch the first one",
            "play this video",
            "play the second one",
            "i want to watch this",
            "lets watch this",
            "play this one",
            "watch the next one",
        ],
    ),
    (
        "recommendations",
        &[
            "show me recommendations for emma",
            "what should emma watch",
            "get recommendations for jake",
            "recommend something for my daughter",
            "suggest videos for my son",
            "what videos would be good for emma",
            "find something for jake to watch",
            "recommendations for my kid",
        ],
    ),
    (
        "download_video",
        &[
            "download video",
            "download this",
            "save this video",
            "save video locally",
        ],
    ),
];

struct Classifier {
    embedder: TextEmbedding,
    intent_embeddings: Vec<(&'static str, Vec<Vec<f32>>)>,
}

static CLASSIFIER: OnceLock<Mutex<Classifier>> = OnceLock::new();

pub fn init_classifier() {
    std::thread::spawn(|| {
        eprintln!("[classifier] loading fastembed model");
        let mut embedder = match TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        ) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[classifier] failed to load: {e}");
                return;
            }
        };

        let mut intent_embeddings = Vec::with_capacity(INTENT_EXAMPLES.len());
        for (intent, examples) in INTENT_EXAMPLES {
            match embedder.embed(examples.to_vec(), None) {
                Ok(embs) => intent_embeddings.push((*intent, embs)),
                Err(e) => eprintln!("[classifier] pre-embed failed for {intent}: {e}"),
            }
        }

        CLASSIFIER
            .set(Mutex::new(Classifier {
                embedder,
                intent_embeddings,
            }))
            .ok();

        eprintln!(
            "[classifier] ready - {} intents pre-embedded",
            INTENT_EXAMPLES.len()
        );
    });
}

pub fn classify(transcript: &str) -> Intent {
    let Some(mutex) = CLASSIFIER.get() else {
        eprintln!("[classifier] not ready - keyword fallback");
        return keyword_fallback(transcript);
    };

    let mut c = mutex.lock().unwrap();

    let t_emb = match c.embedder.embed(vec![transcript], None) {
        Ok(e) => e.into_iter().next().unwrap(),
        Err(e) => {
            eprintln!("[classifier] embed failed: {e}");
            return keyword_fallback(transcript);
        }
    };

    let mut best_intent = "unhandled";
    let mut best_score = 0.0_f32;

    for (intent, example_embs) in &c.intent_embeddings {
        let score = example_embs
            .iter()
            .map(|e| cosine_similarity(&t_emb, e))
            .fold(0.0_f32, f32::max);

        eprintln!("[classifier] {intent:20} score={score:.3}");

        if score > best_score {
            best_score = score;
            best_intent = intent;
        }
    }

    if best_score < MATCH_THRESHOLD {
        eprintln!("[classifier] {best_score:.3} below threshold - unhandled");
        return Intent::Unhandled;
    }

    let intent = Intent::from_str(best_intent);
    eprintln!("[classifier] '{transcript}' - {intent:?} ({best_score:.3})");
    intent
}

pub fn embed_strings(inputs: &[&str]) -> Result<Vec<Vec<f32>>, String> {
    let Some(mutex) = CLASSIFIER.get() else {
        return Err("[classifier] not ready".to_string());
    };
    let mut c = mutex.lock().unwrap();
    c.embedder
        .embed(inputs.to_vec(), None)
        .map_err(|e| format!("[classifier] embed failed: {e}"))
}

fn keyword_fallback(transcript: &str) -> Intent {
    let t = transcript.to_lowercase();
    if t.contains("download") || t.contains("save") {
        return Intent::DownloadVideo;
    }
    if t.contains("search") || t.contains("find") || t.contains("look up") {
        return Intent::Search;
    }
    if t.contains("watch") || t.contains("play") {
        return Intent::WatchVideo;
    }
    if t.contains("assign") || t.contains("give it to") {
        return Intent::AssignVideo;
    }
    if t.contains("recommend") || t.contains("suggest") {
        return Intent::Recommendations;
    }
    if t.contains("add kid") || t.contains("add child") || t.contains("new kid") {
        return Intent::AddKid;
    }
    if t.contains("results") || t.contains("how did") || t.contains("answers") {
        return Intent::KidResults;
    }
    if t.contains("my videos") || t.contains("assigned") {
        return Intent::MyVideos;
    }
    Intent::Unhandled
}
