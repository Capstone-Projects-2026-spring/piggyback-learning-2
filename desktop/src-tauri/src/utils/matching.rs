use crate::utils::text::{cosine_similarity, normalize};
use crate::voice::intent_classifier::embed_strings;

/// Score a kid's transcript against the expected answer.
/// Returns (is_correct, similarity_score).
///
/// Normalization happens first so punctuation differences ("spider." vs "spider")
/// hit the exact-match path at 1.0 rather than going through embeddings.
///
/// Contains-check is intentionally lenient as kids often embed the answer
/// in a longer phrase ("I think it was a spider") which should still count.
pub fn compute_similarity(transcript: &str, expected: &str) -> (bool, f32) {
    let t = normalize(transcript);
    let e = normalize(expected);

    if t == e {
        eprintln!("[matching] exact match");
        return (true, 1.0);
    }

    if t.contains(&e) || e.contains(&t) {
        eprintln!("[matching] contains match");
        return (true, 1.0);
    }

    match embed_strings(&[&t, &e]) {
        Ok(embs) if embs.len() == 2 => {
            let score = cosine_similarity(&embs[0], &embs[1]);
            eprintln!("[matching] cosine={score:.3}");
            (score >= 0.6, score)
        }
        Ok(_) => {
            eprintln!("[matching] unexpected embedding count - scoring as incorrect");
            (false, 0.0)
        }
        Err(err) => {
            eprintln!("[matching] embed failed ({err}) - exact-only fallback");
            (
                t == e || t.contains(&e),
                if t == e || t.contains(&e) { 1.0 } else { 0.0 },
            )
        }
    }
}
