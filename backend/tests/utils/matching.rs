use crate::utils::voice::matching::{compute_similarity, normalize_text};

#[test]
fn normalize_text() {
    assert_eq!(normalize_text("Hello, Kid!"), "hello kid");
    assert_eq!(normalize_text("  Space... 123  "), "space 123");
}

#[test]
fn compute_similarity_exact() {
    let (correct, score) = compute_similarity("Mars", "mars");
    assert!(correct);
    assert_eq!(score, 1.0);
}

#[test]
fn compute_similarity_fuzzy() {
    let (correct, score) = compute_similarity("planit", "planet");
    assert!(correct);
    assert!(score > 0.7);
}