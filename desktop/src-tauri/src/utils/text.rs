/// Strip non-alphanumeric characters, collapse whitespace, lowercase.
/// Used by the wake word detector, answer similarity scorer, and tag inference.
pub fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Cosine similarity between two equal-length float vectors.
/// Returns 0.0 if either vector is zero-length or all zeros.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

const NOISE_TRANSCRIPTS: &[&str] = &[
    "you",
    "the",
    "a",
    "uh",
    "um",
    "oh",
    "ah",
    "hm",
    "hmm",
    "thank you",
    "thanks",
    "bye",
    "okay",
    "ok",
];

/// Returns true if the transcript is likely noise, a filler word, or too short
/// to contain a real utterance. Used to gate the voice pipeline before classification.
pub fn is_noise_transcript(transcript: &str) -> bool {
    let t = transcript.trim().to_lowercase();
    if t.split_whitespace().count() == 1 && NOISE_TRANSCRIPTS.contains(&t.as_str()) {
        return true;
    }
    t.chars().filter(|c| c.is_alphabetic()).count() < 3
}
