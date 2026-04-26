use webrtc_vad::{SampleRate, Vad, VadMode};

/// ~700ms of silence at 30ms frames triggers a flush
const SILENCE_FRAMES_THRESHOLD: usize = 23;
/// 15s max before forced flush
const MAX_SAMPLES: usize = 16000 * 15;
/// 0.5s minimum as shorter chunks are likely noise, not real utterances
const MIN_SPEECH_SAMPLES: usize = 16000 / 2;
/// 30ms frame at 16kHz
const FRAME_SIZE: usize = 480;

pub struct VadChunker {
    vad: Vad,
    speech_buf: Vec<f32>,
    silent_frames: usize,
    has_speech: bool,
}

impl VadChunker {
    pub fn new() -> Self {
        Self {
            vad: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Aggressive),
            speech_buf: Vec::new(),
            silent_frames: 0,
            has_speech: false,
        }
    }

    /// Feed mono f32 samples at 16kHz.
    /// Returns `Some(chunk)` when a complete utterance is ready for transcription.
    /// Trailing silence is preserved so Whisper gets natural sentence endings.
    pub fn push(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        let mut i = 0;

        while i + FRAME_SIZE <= samples.len() {
            let frame = &samples[i..i + FRAME_SIZE];
            i += FRAME_SIZE;

            let i16_frame: Vec<i16> = frame
                .iter()
                .map(|&s| (s * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16)
                .collect();

            let is_speech = self.vad.is_voice_segment(&i16_frame).unwrap_or(false);

            if is_speech {
                self.has_speech = true;
                self.silent_frames = 0;
                self.speech_buf.extend_from_slice(frame);
            } else if self.has_speech {
                self.silent_frames += 1;
                self.speech_buf.extend_from_slice(frame);

                if self.silent_frames >= SILENCE_FRAMES_THRESHOLD {
                    return self.flush("silence threshold");
                }
            }

            if self.speech_buf.len() >= MAX_SAMPLES {
                return self.flush("max duration");
            }
        }

        None
    }

    fn flush(&mut self, reason: &str) -> Option<Vec<f32>> {
        if self.speech_buf.len() < MIN_SPEECH_SAMPLES {
            eprintln!("[vad] chunk too short, discarding ({reason})");
            self.reset();
            return None;
        }
        eprintln!(
            "[vad] flush ({reason}) - {:.1}s",
            self.speech_buf.len() as f32 / 16000.0
        );
        let chunk = self.speech_buf.clone();
        self.reset();
        Some(chunk)
    }

    fn reset(&mut self) {
        self.speech_buf.clear();
        self.silent_frames = 0;
        self.has_speech = false;
    }
}
