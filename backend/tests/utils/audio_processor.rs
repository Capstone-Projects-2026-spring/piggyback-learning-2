use crate::utils::voice::audio_processor::{parse_wav};

#[test]
fn parse_wav_invalid_header() {
    // Method: parse_wav
    // Should fail with a "bad header" if we pass random bytes
    let result = parse_wav(&[1, 2, 3, 4]);
    assert!(result.is_err());
}