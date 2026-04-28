use nnnoiseless::DenoiseState;

const PRE_EMPHASIS: f32 = 0.97;

/// Convert multi-channel interleaved samples to mono by averaging channels.
pub fn to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    data.chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Resample from `from` Hz to `to` Hz using linear interpolation.
/// Returns the input unchanged if rates are equal.
pub fn resample(samples: Vec<f32>, from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return samples;
    }
    let ratio = from as f64 / to as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = pos as usize;
            let frac = (pos - idx as f64) as f32;
            let s0 = samples[idx.min(samples.len() - 1)];
            let s1 = samples[(idx + 1).min(samples.len() - 1)];
            s0 + (s1 - s0) * frac
        })
        .collect()
}

/// Denoise audio using RNNoise. Runs frame by frame at 480 samples per frame.
/// Should be called on 16kHz mono audio before VAD or transcription.
pub fn denoise(samples: &[f32]) -> Vec<f32> {
    let frame_size = DenoiseState::FRAME_SIZE; // 480 samples
    let mut state = DenoiseState::new();
    let mut out = vec![0.0f32; samples.len()];

    for (in_chunk, out_chunk) in samples.chunks(frame_size).zip(out.chunks_mut(frame_size)) {
        let mut frame = [0.0f32; 480];
        frame[..in_chunk.len()].copy_from_slice(in_chunk);
        let mut denoised = [0.0f32; 480];
        state.process_frame(&mut denoised, &frame);
        out_chunk.copy_from_slice(&denoised[..out_chunk.len()]);
    }
    out
}

/// Process a mono 16kHz chunk for STT input:
/// denoise -> pre-emphasis -> peak normalize.
/// Returns empty vec if the chunk is silent after denoising.
pub fn process_f32(samples: Vec<f32>) -> Vec<f32> {
    if samples.is_empty() {
        return vec![];
    }

    let denoised = denoise(&samples);

    // Pre-emphasis - boost high frequencies for cleaner STT input
    let mut emphasized = Vec::with_capacity(denoised.len());
    emphasized.push(denoised[0]);
    for i in 1..denoised.len() {
        emphasized.push(denoised[i] - PRE_EMPHASIS * denoised[i - 1]);
    }

    // Peak normalize
    let max = emphasized.iter().map(|s| s.abs()).fold(0_f32, f32::max);
    if max < 1e-6 {
        eprintln!("[audio] rejected - silent after denoising");
        return vec![];
    }

    emphasized.iter().map(|s| s / max).collect()
}
