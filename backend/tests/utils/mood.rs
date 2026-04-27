use crate::utils::voice::mood::{detect_mood, rms_energy};

#[test]
fn rms_energy() {
    let silence = vec![0, 0, 0];
    assert_eq!(rms_energy(&silence), 0.0);
    
    let noise = vec![1000, 1000, 1000];
    assert!(rms_energy(&noise) > 0.0);
}

#[test]
fn detect_mood_categories() {
    // Bored threshold < 2500
    let (mood_b, _) = detect_mood(&vec![500, 500]);
    assert_eq!(mood_b, "bored");

    // Excited threshold > 4500
    let (mood_e, _) = detect_mood(&vec![10000, 10000]);
    assert_eq!(mood_e, "excited");
}

#[test]
fn parse_wav_invalid_header() {
    // Fails with a "bad header" if we pass random bytes
    let result = parse_wav(&[1, 2, 3, 4]);
    assert!(result.is_err());
}