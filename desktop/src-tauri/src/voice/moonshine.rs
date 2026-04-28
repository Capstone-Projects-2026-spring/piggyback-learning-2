use ndarray::{Array1, Array2, Array3, ArrayD, IxDyn};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::{DynValue, Tensor};
use std::borrow::Cow;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tokenizers::Tokenizer;

// Model sessions
static PREPROCESS: OnceLock<Mutex<Session>> = OnceLock::new();
static ENCODE: OnceLock<Mutex<Session>> = OnceLock::new();
static UNCACHED_DECODE: OnceLock<Mutex<Session>> = OnceLock::new();
static CACHED_DECODE: OnceLock<Mutex<Session>> = OnceLock::new();
static TOKENIZER: OnceLock<Tokenizer> = OnceLock::new();

// Constants
const SOT: i32 = 1;
const EOT: i32 = 2;
const VOCAB_SIZE: usize = 32768;

/// Max 6.5 tokens per second - prevents hallucination loops
const TOKENS_PER_SECOND: f32 = 6.5;
const MIN_TOKENS: usize = 10;

/// Number of transformer layers - base=8
const N_LAYERS: usize = 8;

/// Base names for uncached_decode KV cache outputs, one per layer
const UNCACHED_BASES: [&str; N_LAYERS] = [
    "functional_28",
    "functional_31",
    "functional_34",
    "functional_37",
    "functional_40",
    "functional_43",
    "functional_46",
    "functional_49",
];

/// (base_name, input_layer_index) for cached_decode KV cache outputs
const CACHED_BASES: [(&str, usize); N_LAYERS] = [
    ("functional_29", 132),
    ("functional_32", 136),
    ("functional_35", 140),
    ("functional_38", 144),
    ("functional_41", 148),
    ("functional_44", 152),
    ("functional_47", 156),
    ("functional_50", 160),
];

fn load_session(path: &Path) -> Mutex<Session> {
    Mutex::new(
        Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(path)
            .unwrap_or_else(|e| panic!("[moonshine] failed to load {path:?}: {e}")),
    )
}

pub fn init_moonshine(model_dir: &Path) {
    let load = |name: &str| {
        eprintln!("[moonshine] loading {name}");
        load_session(&model_dir.join(format!("{name}.onnx")))
    };

    PREPROCESS.get_or_init(|| load("preprocess"));
    ENCODE.get_or_init(|| load("encode"));
    UNCACHED_DECODE.get_or_init(|| load("uncached_decode"));
    CACHED_DECODE.get_or_init(|| load("cached_decode"));
    TOKENIZER.get_or_init(|| {
        eprintln!("[moonshine] loading tokenizer");
        Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .expect("[moonshine] failed to load tokenizer")
    });

    eprintln!("[moonshine] ready");
}

/// Transcribe raw 16kHz mono f32 samples.
/// Returns an empty string on any failure.
pub fn transcribe(samples: &[f32]) -> String {
    run(samples).unwrap_or_default()
}

fn run(samples: &[f32]) -> Option<String> {
    let (features_shape, features_data) = preprocess(samples).or_else(|| {
        eprintln!("[moonshine] preprocess failed");
        None
    })?;

    let (encoded_shape, encoded_data) = encode(&features_shape, features_data).or_else(|| {
        eprintln!("[moonshine] encode failed");
        None
    })?;

    let (first_token, mut kv_cache, mut kv_shapes) = uncached_decode(&encoded_shape, &encoded_data)
        .or_else(|| {
            eprintln!("[moonshine] uncached_decode failed");
            None
        })?;

    let max_tokens = ((samples.len() as f32 / 16000.0) * TOKENS_PER_SECOND) as usize;
    let max_tokens = max_tokens.max(MIN_TOKENS);

    let mut generated = vec![first_token];
    let mut current = first_token;
    let cache_names = cached_output_names();

    for step in 0..max_tokens {
        if current == EOT {
            eprintln!("[moonshine] EOT at step {step}");
            break;
        }
        let (next, new_cache, new_shapes) = cached_decode(
            current,
            &encoded_shape,
            &encoded_data,
            &kv_cache,
            &kv_shapes,
            &cache_names,
        )
        .or_else(|| {
            eprintln!("[moonshine] cached_decode failed at step {step}");
            None
        })?;
        current = next;
        generated.push(current);
        kv_cache = new_cache;
        kv_shapes = new_shapes;
    }

    let text = decode_tokens(&generated);
    eprintln!("[moonshine] transcript: {text:?}");
    Some(text)
}

// Pipeline stages

fn preprocess(samples: &[f32]) -> Option<(Vec<usize>, Vec<f32>)> {
    let audio = Array2::from_shape_vec((1, samples.len()), samples.to_vec()).ok()?;
    let audio_tensor = Tensor::from_array(audio).ok()?;

    let mut session = PREPROCESS.get()?.lock().ok()?;
    let outputs = session.run(ort::inputs!["args_0" => audio_tensor]).ok()?;
    let t = outputs["sequential"].try_extract_tensor::<f32>().ok()?;
    Some((
        t.0.iter().map(|&d| d as usize).collect(),
        t.1.iter().copied().collect(),
    ))
}

fn encode(features_shape: &[usize], features_data: Vec<f32>) -> Option<(Vec<usize>, Vec<f32>)> {
    // Hidden dim is read from the preprocess output shape - works for both tiny (288) and base (416)
    let hidden_dim = features_shape[2];
    let enc_input = Array3::from_shape_vec(
        (features_shape[0], features_shape[1], hidden_dim),
        features_data,
    )
    .ok()?;
    let seq_len_arr = Array1::from_vec(vec![features_shape[1] as i32]);
    let enc_tensor = Tensor::from_array(enc_input).ok()?;
    let seq_tensor = Tensor::from_array(seq_len_arr).ok()?;

    let mut session = ENCODE.get()?.lock().ok()?;
    let outputs = session
        .run(ort::inputs![
            "args_0" => enc_tensor,
            "args_1" => seq_tensor
        ])
        .ok()?;
    let t = outputs["layer_normalization_16"]
        .try_extract_tensor::<f32>()
        .ok()?;
    Some((
        t.0.iter().map(|&d| d as usize).collect(),
        t.1.iter().copied().collect(),
    ))
}

fn uncached_decode(
    encoded_shape: &[usize],
    encoded_data: &[f32],
) -> Option<(i32, Vec<Vec<f32>>, Vec<Vec<usize>>)> {
    let hidden_dim = encoded_shape[2];
    let tok_tensor = Tensor::from_array(Array2::from_shape_vec((1, 1), vec![SOT]).ok()?).ok()?;
    let enc_tensor = Tensor::from_array(
        Array3::from_shape_vec(
            (encoded_shape[0], encoded_shape[1], hidden_dim),
            encoded_data.to_vec(),
        )
        .ok()?,
    )
    .ok()?;
    let seq_tensor = Tensor::from_array(Array1::from_vec(vec![encoded_shape[1] as i32])).ok()?;

    let mut session = UNCACHED_DECODE.get()?.lock().ok()?;
    let outputs = session
        .run(ort::inputs![
            "args_0" => tok_tensor,
            "args_1" => enc_tensor,
            "args_2" => seq_tensor
        ])
        .ok()?;

    let logits = outputs["reversible_embedding"]
        .try_extract_tensor::<f32>()
        .ok()?;
    let first_token = argmax(logits.1.iter().copied())?;
    eprintln!("[moonshine] first_token={first_token}");

    let (kv_cache, kv_shapes) = extract_kv_cache(&outputs, &uncached_output_names())?;
    Some((first_token, kv_cache, kv_shapes))
}

fn cached_decode(
    current_token: i32,
    encoded_shape: &[usize],
    encoded_data: &[f32],
    kv_cache: &[Vec<f32>],
    kv_shapes: &[Vec<usize>],
    cache_names: &[String],
) -> Option<(i32, Vec<Vec<f32>>, Vec<Vec<usize>>)> {
    let hidden_dim = encoded_shape[2];
    let tok_t: DynValue =
        Tensor::from_array(Array2::from_shape_vec((1, 1), vec![current_token]).ok()?)
            .ok()?
            .into();
    let enc_t: DynValue = Tensor::from_array(
        Array3::from_shape_vec(
            (encoded_shape[0], encoded_shape[1], hidden_dim),
            encoded_data.to_vec(),
        )
        .ok()?,
    )
    .ok()?
    .into();
    let seq_t: DynValue = Tensor::from_array(Array1::from_vec(vec![encoded_shape[1] as i32]))
        .ok()?
        .into();

    let mut inputs: Vec<(Cow<'static, str>, DynValue)> = vec![
        (Cow::Borrowed("args_0"), tok_t),
        (Cow::Borrowed("args_1"), enc_t),
        (Cow::Borrowed("args_2"), seq_t),
    ];

    for (i, (data, shape)) in kv_cache.iter().zip(kv_shapes.iter()).enumerate() {
        let arr = ArrayD::from_shape_vec(IxDyn(shape), data.clone()).ok()?;
        let t: DynValue = Tensor::from_array(arr).ok()?.into();
        inputs.push((Cow::Owned(format!("args_{}", i + 3)), t));
    }

    let mut session = CACHED_DECODE.get()?.lock().ok()?;
    let outputs = session.run(inputs).ok()?;

    let logits = outputs["reversible_embedding"]
        .try_extract_tensor::<f32>()
        .ok()?;
    let next_token = argmax(logits.1.iter().copied())?;

    let (new_cache, new_shapes) = extract_kv_cache(&outputs, cache_names)?;
    Some((next_token, new_cache, new_shapes))
}

// Helpers
fn extract_kv_cache(
    outputs: &ort::session::SessionOutputs,
    names: &[String],
) -> Option<(Vec<Vec<f32>>, Vec<Vec<usize>>)> {
    let mut cache = Vec::with_capacity(names.len());
    let mut shapes = Vec::with_capacity(names.len());
    for name in names {
        let t = outputs[name.as_str()].try_extract_tensor::<f32>().ok()?;
        shapes.push(t.0.iter().map(|&d| d as usize).collect());
        cache.push(t.1.iter().copied().collect());
    }
    Some((cache, shapes))
}

fn argmax(iter: impl Iterator<Item = f32>) -> Option<i32> {
    let all: Vec<f32> = iter.collect();
    let last = &all[all.len().saturating_sub(VOCAB_SIZE)..];
    last.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i as i32)
}

fn uncached_output_names() -> Vec<String> {
    UNCACHED_BASES
        .iter()
        .flat_map(|base| {
            [
                base.to_string(),
                format!("{base}_1"),
                format!("{base}_2"),
                format!("{base}_3"),
            ]
        })
        .collect()
}

fn cached_output_names() -> Vec<String> {
    CACHED_BASES
        .iter()
        .flat_map(|(base, layer)| {
            [
                base.to_string(),
                format!("{base}_1"),
                format!("input_layer_{layer}"),
                format!("input_layer_{}", layer + 1),
            ]
        })
        .collect()
}

fn decode_tokens(tokens: &[i32]) -> String {
    let tokenizer = match TOKENIZER.get() {
        Some(t) => t,
        None => return String::new(),
    };
    let ids: Vec<u32> = tokens
        .iter()
        .copied()
        .filter(|&t| t != SOT && t != EOT && t >= 0)
        .map(|t| t as u32)
        .collect();
    tokenizer
        .decode(&ids, true)
        .unwrap_or_default()
        .trim()
        .to_lowercase()
}
