use strsim::levenshtein;

pub fn compute_similarity(user: &str, expected: &str) -> (bool, f32) {
    let u = user.to_lowercase();
    let e = expected.to_lowercase();

    for token in u.split_whitespace() {
        if token == e {
            return (true, 1.0);
        }
    }

    let mut best_score = 0.0_f32;
    for token in u.split_whitespace() {
        let dist = levenshtein(token, &e);
        let max_len = token.len().max(e.len());
        if max_len > 0 {
            let score = 1.0 - (dist as f32 / max_len as f32);
            best_score = best_score.max(score);
        }
    }

    let is_correct = best_score > 0.8;

    (is_correct, best_score)
}
