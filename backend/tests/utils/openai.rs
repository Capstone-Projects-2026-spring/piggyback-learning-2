use crate::utils::openai::{build_prompt, build_transcript, sample_frames};
use crate::models::_entities::frames;

#[test]
fn sample_frames_logic() {
    let mut mock_frames = Vec::new();

    for i in 0..10 {
        mock_frames.push(frames::Model {
            id: i,
            video_id: "vid".to_string(),
            file_path: format!("path/{}", i),
            ..Default::default()
        });
    }

    let sampled = sample_frames(&mock_frames, 5);
    assert_eq!(sampled.len(), 5);
}

#[test]
fn build_transcript_formatting() {
    let mock_frames = vec![
        frames::Model {
            timestamp_formatted: "00:01".to_string(),
            subtitle_text: Some("Hello".to_string()),
            ..Default::default()
        },
        frames::Model {
            timestamp_formatted: "00:02".to_string(),
            subtitle_text: Some("World".to_string()),
            ..Default::default()
        }
    ];

    let transcript = build_transcript(&mock_frames);
    assert!(transcript.contains("[00:01] Hello"));
    assert!(transcript.contains("[00:02] World"));
}

#[test]
fn build_prompt_structure() {
    let prompt = build_prompt("test transcript", 60, 0, 60, &vec!["old question".to_string()]);
    
    assert!(prompt.contains("test transcript"));
    assert!(prompt.contains("PREVIOUSLY ASKED QUESTIONS"));
    assert!(prompt.contains("old question"));
}