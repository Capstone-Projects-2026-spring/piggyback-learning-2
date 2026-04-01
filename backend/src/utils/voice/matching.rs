use strsim::levenshtein;
use thesaurus;

pub fn normalize_text(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

pub fn compute_similarity(user: &str, expected: &str) -> (bool, f32) {
    let u = normalize_text(user);
    let e = normalize_text(expected);

    let user_tokens: Vec<&str> = u.split_whitespace().collect();

    if user_tokens.iter().any(|&t| t == e) {
        return (true, 1.0);
    }

    let expected_syns: Vec<String> = thesaurus::synonyms(&e);

    for token in &user_tokens {
        if expected_syns.iter().any(|syn| syn == token) {
            return (true, 0.9);
        }

        let token_syns: Vec<String> = thesaurus::synonyms(token);

        if token_syns.iter().any(|syn| syn == &e) {
            return (true, 0.9);
        }
    }

    // Fuzzy token match fallback (Levenshtein)
    let mut best_score = 0.0_f32;
    for token in user_tokens {
        let dist = levenshtein(token, &e);
        let max_len = token.len().max(e.len());
        if max_len > 0 {
            let score = 1.0 - (dist as f32 / max_len as f32);
            if score > best_score {
                best_score = score;
            }
        }
    }

    let is_correct = best_score >= 0.6;
    (is_correct, best_score)
}
