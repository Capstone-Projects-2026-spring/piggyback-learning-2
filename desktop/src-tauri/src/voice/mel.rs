use ndarray::Array2;
use rustfft::{num_complex::Complex, FftPlanner};

const SAMPLE_RATE: f32 = 16000.0;
const N_FFT: usize = 512;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;
const F_MIN: f32 = 20.0;

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

fn build_mel_filterbank(n_fft: usize, n_mels: usize, sr: f32, f_min: f32) -> Array2<f32> {
    let f_max = sr / 2.0;
    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);
    let n_freqs = n_fft / 2 + 1;

    let mel_points: Vec<f32> = (0..=n_mels + 1)
        .map(|i| mel_to_hz(mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32))
        .collect();

    let bin_points: Vec<usize> = mel_points
        .iter()
        .map(|&f| ((n_fft + 1) as f32 * f / sr).floor() as usize)
        .collect();

    let mut fb = Array2::<f32>::zeros((n_mels, n_freqs));

    for m in 0..n_mels {
        let f_left = bin_points[m];
        let f_center = bin_points[m + 1];
        let f_right = bin_points[m + 2];

        for k in f_left..f_center {
            if f_center > f_left && k < n_freqs {
                fb[[m, k]] = (k - f_left) as f32 / (f_center - f_left) as f32;
            }
        }
        for k in f_center..f_right {
            if f_right > f_center && k < n_freqs {
                fb[[m, k]] = (f_right - k) as f32 / (f_right - f_center) as f32;
            }
        }
    }

    fb
}

fn hann_window(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos()))
        .collect()
}

/// Compute log mel filterbank features from raw 16kHz mono samples.
/// Returns None if the audio is too short to produce meaningful features.
pub fn compute_fbank(samples: &[f32]) -> Option<Array2<f32>> {
    use ndarray::Axis;

    if samples.len() < N_FFT {
        return None;
    }

    let window = hann_window(N_FFT);
    let n_freqs = N_FFT / 2 + 1;
    let filterbank = build_mel_filterbank(N_FFT, N_MELS, SAMPLE_RATE, F_MIN);
    let fft = FftPlanner::<f32>::new().plan_fft_forward(N_FFT);

    let n_frames = (samples.len() - N_FFT) / HOP_LENGTH + 1;
    if n_frames < 5 {
        eprintln!("[mel] too few frames: {n_frames}");
        return None;
    }

    let mut fbank = Array2::<f32>::zeros((n_frames, N_MELS));

    for (fi, start) in (0..=samples.len() - N_FFT)
        .step_by(HOP_LENGTH)
        .enumerate()
        .take(n_frames)
    {
        let mut frame: Vec<Complex<f32>> = samples[start..start + N_FFT]
            .iter()
            .zip(&window)
            .map(|(&s, &w)| Complex { re: s * w, im: 0.0 })
            .collect();

        fft.process(&mut frame);

        let power: Vec<f32> = frame[..n_freqs]
            .iter()
            .map(|c| c.re * c.re + c.im * c.im)
            .collect();

        for m in 0..N_MELS {
            let energy: f32 = (0..n_freqs).map(|k| filterbank[[m, k]] * power[k]).sum();
            fbank[[fi, m]] = energy.max(1e-10_f32).ln();
        }
    }

    // Mean-variance normalisation per mel bin
    let mean = fbank.mean_axis(Axis(0)).unwrap();
    let std = fbank.std_axis(Axis(0), 0.0);
    fbank = (fbank - &mean) / (std + 1e-8);

    eprintln!("[mel] fbank shape: [{n_frames}, {N_MELS}]");
    Some(fbank)
}
