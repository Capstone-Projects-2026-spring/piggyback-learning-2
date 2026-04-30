use crate::voice::state::get_silero;
use ndarray::{Array2, Array3};
use ort::value::Tensor;

/// ~256ms of silence at 32ms frames triggers a flush
const SILENCE_FRAMES_THRESHOLD: usize = 8;
/// 15s max before forced flush
const MAX_SAMPLES: usize = 16000 * 15;
/// 0.5s minimum - shorter chunks are likely noise, not real utterances
const MIN_SPEECH_SAMPLES: usize = 16000 / 2;
/// Silero VAD frame size at 16kHz (512 samples = 32ms)
const FRAME_SIZE: usize = 512;
/// Probability above which a frame is considered speech
const SPEECH_THRESHOLD: f32 = 0.15;

pub struct VadChunker {
    speech_buf: Vec<f32>,
    silent_frames: usize,
    has_speech: bool,
    /// Silero RNN state - shape [2, 1, 128]
    state: Vec<f32>,
}

impl VadChunker {
    pub fn new() -> Self {
        Self {
            speech_buf: Vec::new(),
            silent_frames: 0,
            has_speech: false,
            state: vec![0.0f32; 2 * 1 * 128],
        }
    }

    /// Feed mono f32 samples at 16kHz.
    /// Returns `Some(chunk)` when a complete utterance is ready for transcription.
    pub fn push(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        let mut i = 0;
        while i + FRAME_SIZE <= samples.len() {
            let frame = &samples[i..i + FRAME_SIZE];
            i += FRAME_SIZE;

            let is_speech = self.is_speech(frame).unwrap_or(false);

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

    /// Run a single 512-sample frame through Silero VAD.
    /// Updates the RNN hidden state in place and returns speech probability.
    fn is_speech(&mut self, frame: &[f32]) -> Option<bool> {
        let mut session = match get_silero().lock() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[vad] lock failed: {e}");
                return None;
            }
        };

        let input = match Array2::from_shape_vec((1, FRAME_SIZE), frame.to_vec()) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[vad] input shape failed: {e}");
                return None;
            }
        };

        let state_tensor = match Array3::from_shape_vec((2, 1, 128), self.state.clone()) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[vad] state shape failed: {e}");
                return None;
            }
        };

        // sr is a scalar - shape []
        let sr_array = ndarray::arr0(16000i64);

        let input_tensor = match Tensor::from_array(input) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[vad] input tensor failed: {e}");
                return None;
            }
        };
        let state_input = match Tensor::from_array(state_tensor) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[vad] state tensor failed: {e}");
                return None;
            }
        };
        let sr_tensor = match Tensor::from_array(sr_array) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[vad] sr tensor failed: {e}");
                return None;
            }
        };

        let outputs = match session.run(ort::inputs![
            "input" => input_tensor,
            "state" => state_input,
            "sr"    => sr_tensor
        ]) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[vad] inference failed: {e}");
                return None;
            }
        };

        let prob = match outputs["output"].try_extract_tensor::<f32>() {
            Ok(t) => t.1.iter().next().copied()?,
            Err(e) => {
                eprintln!("[vad] output extract failed: {e}");
                return None;
            }
        };

        self.state = match outputs["stateN"].try_extract_tensor::<f32>() {
            Ok(t) => t.1.iter().copied().collect(),
            Err(e) => {
                eprintln!("[vad] stateN extract failed: {e}");
                return None;
            }
        };

        Some(prob >= SPEECH_THRESHOLD)
    }

    pub fn flush(&mut self, reason: &str) -> Option<Vec<f32>> {
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

    pub fn reset(&mut self) {
        self.speech_buf.clear();
        self.silent_frames = 0;
        self.has_speech = false;
        self.state = vec![0.0f32; 2 * 1 * 128];
    }
}
