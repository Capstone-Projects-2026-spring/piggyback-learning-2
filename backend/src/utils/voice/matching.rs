use strsim::levenshtein;

pub fn normalize_text(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

pub fn compute_similarity(user: &str, expected: &str) -> (bool, f32) {
    let u = normalize_text(user);
    let e = normalize_text(expected);

    if u.split_whitespace().any(|word| word == e) {
        return (true, 0.95);
    }

    if u == e {
        return (true, 1.0);
    }

    let dist = levenshtein(&u, &e);
    let max_len = u.len().max(e.len());

    if max_len == 0 {
        return (false, 0.0);
    }

    let similarity = 1.0 - (dist as f32 / max_len as f32);

    let is_correct = similarity > 0.8;

    (is_correct, similarity)
}
