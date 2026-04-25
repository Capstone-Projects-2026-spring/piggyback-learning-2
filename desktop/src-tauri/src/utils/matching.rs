use crate::utils::voice::intent_classifier::embed_strings;

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

pub fn compute_similarity(transcript: &str, expected: &str) -> (bool, f32) {
    let t = normalize(transcript);
    let e = normalize(expected);

    // Exact match after normalization — no need to run embeddings
    if t == e || t.contains(&e) || e.contains(&t) {
        eprintln!("[matching] exact match after normalize");
        return (true, 1.0);
    }

    match embed_strings(&[&t, &e]) {
        Ok(embs) if embs.len() == 2 => {
            let score = cosine_similarity(&embs[0], &embs[1]);
            eprintln!("[matching] similarity={score:.3}");
            (score >= 0.6, score)
        }
        Ok(_) => (false, 0.0),
        Err(e) => {
            eprintln!("[matching] embed failed: {e} — falling back to exact");
            let exact = t == e || t.contains(&e);
            (exact, if exact { 1.0 } else { 0.0 })
        }
    }
}
