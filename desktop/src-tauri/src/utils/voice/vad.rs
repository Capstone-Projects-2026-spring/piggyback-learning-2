use webrtc_vad::{SampleRate, Vad, VadMode};

/// ~700ms of silence at 30ms frames triggers a flush
const SILENCE_FRAMES_THRESHOLD: usize = 23;
/// 15s max before forced flush
const MAX_SAMPLES: usize = 16000 * 15;
/// 0.5s minimum before we bother flushing
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
        let vad = Vad::new_with_rate_and_mode(webrtc_vad::SampleRate::Rate16kHz, VadMode::Quality);
        Self {
            vad,
            speech_buf: Vec::new(),
            silent_frames: 0,
            has_speech: false,
        }
    }

    /// Feed mono f32 samples at 16kHz.
    /// Returns Some(chunk) when a complete utterance is detected.
    pub fn push(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        let mut result = None;
        let mut i = 0;

        while i + FRAME_SIZE <= samples.len() {
            let frame = &samples[i..i + FRAME_SIZE];

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
                // Keep trailing silence so whisper gets natural endings
                self.speech_buf.extend_from_slice(frame);

                if self.silent_frames >= SILENCE_FRAMES_THRESHOLD {
                    if self.speech_buf.len() >= MIN_SPEECH_SAMPLES {
                        eprintln!(
                            "[vad] utterance end detected ({:.1}s)",
                            self.speech_buf.len() as f32 / 16000.0
                        );
                        result = Some(self.speech_buf.clone());
                    } else {
                        eprintln!("[vad] chunk too short, discarding");
                    }
                    self.reset();
                }
            }

            if self.speech_buf.len() >= MAX_SAMPLES {
                eprintln!("[vad] max duration reached, forcing flush");
                result = Some(self.speech_buf.clone());
                self.reset();
            }

            i += FRAME_SIZE;
        }

        result
    }

    fn reset(&mut self) {
        self.speech_buf.clear();
        self.silent_frames = 0;
        self.has_speech = false;
    }
}
