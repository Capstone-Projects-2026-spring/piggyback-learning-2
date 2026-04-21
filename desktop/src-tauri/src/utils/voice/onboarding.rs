use super::enrollment::{emit_enrollment, EnrollmentEvent, ENROLLMENT_PROMPTS};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

const REJECTED_TRANSCRIPTS: &[&str] = &[
    "[blank_audio]",
    "[silence]",
    "[noise]",
    "(blank)",
    "(silence)",
    "blank audio",
    "thank you",
    "thanks",
    "...",
    ".",
];

fn clean_transcript(text: &str) -> String {
    let mut result = String::new();
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => result.push(c),
            _ => {}
        }
    }
    result
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn is_valid_name(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return false;
    }
    if REJECTED_TRANSCRIPTS.iter().any(|r| t.as_str() == *r) {
        return false;
    }
    if !t.chars().any(|c| c.is_alphabetic()) {
        return false;
    }
    if t.split_whitespace().count() > 4 {
        return false;
    }
    true
}

fn capitalise_words(text: &str) -> String {
    text.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnboardingFlow {
    Parent,
    Kid,
}

impl OnboardingFlow {
    pub fn role(&self) -> &'static str {
        match self {
            OnboardingFlow::Parent => "parent",
            OnboardingFlow::Kid => "kid",
        }
    }

    fn flow_name(&self) -> &'static str {
        match self {
            OnboardingFlow::Parent => "parent",
            OnboardingFlow::Kid => "kid",
        }
    }

    fn stage(&self, event: &str) -> String {
        match self {
            OnboardingFlow::Parent => event.to_string(),
            OnboardingFlow::Kid => format!("kid_{event}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnboardingStage {
    Idle,
    WaitingForName,
    CollectingVoice { prompt_index: usize },
    Done,
}

#[derive(Debug, Clone)]
pub struct OnboardingState {
    pub stage: OnboardingStage,
    pub flow: OnboardingFlow,
    pub name: Option<String>,
    pub embeddings: Vec<Vec<f32>>,
}

impl OnboardingState {
    pub fn new() -> Self {
        Self {
            stage: OnboardingStage::Idle,
            flow: OnboardingFlow::Parent,
            name: None,
            embeddings: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.stage != OnboardingStage::Idle && self.stage != OnboardingStage::Done
    }
}

pub type SharedOnboarding = Arc<Mutex<OnboardingState>>;

pub fn new_onboarding() -> SharedOnboarding {
    Arc::new(Mutex::new(OnboardingState::new()))
}

pub fn start(app: &AppHandle, onboarding: &SharedOnboarding, flow: OnboardingFlow) {
    {
        let mut o = onboarding.lock().unwrap();
        *o = OnboardingState::new();
        o.flow = flow;
        o.stage = OnboardingStage::WaitingForName;
    }

    let o = onboarding.lock().unwrap();
    let message = match o.flow {
        OnboardingFlow::Parent => {
            "Hi there! I'm Peppa, your learning buddy. Let's get started — what's your name?"
                .to_string()
        }
        OnboardingFlow::Kid => {
            "Hi! Let's set up a new kid account. Kid, what's your name?".to_string()
        }
    };

    emit_enrollment(
        app,
        EnrollmentEvent {
            stage: o.flow.stage("greet"),
            message,
            prompt_index: 0,
            total_prompts: ENROLLMENT_PROMPTS.len(),
            prompts: ENROLLMENT_PROMPTS.iter().map(|s| s.to_string()).collect(),
            flow: o.flow.flow_name().to_string(),
        },
    );

    eprintln!(
        "[onboarding] started — flow={} waiting for name",
        o.flow.role()
    );
}

pub fn try_set_name(app: &AppHandle, onboarding: &SharedOnboarding, transcript: &str) -> bool {
    let cleaned = clean_transcript(transcript);
    if !is_valid_name(&cleaned) {
        eprintln!("[onboarding] rejected name: '{transcript}' → cleaned='{cleaned}'");
        return false;
    }

    let name = capitalise_words(&cleaned);
    eprintln!("[onboarding] accepted name: '{name}'");

    let mut o = onboarding.lock().unwrap();
    o.name = Some(name.clone());

    let message = match o.flow {
        OnboardingFlow::Parent => format!(
            "Nice to meet you, {name}! Now I need to learn your voice so I can recognise you. \
             Read each sentence below out loud clearly."
        ),
        OnboardingFlow::Kid => format!(
            "Great, {name}! Now I need to learn your voice. \
             Read each sentence below out loud clearly."
        ),
    };

    emit_enrollment(
        app,
        EnrollmentEvent {
            stage: o.flow.stage("name_confirmed"),
            message,
            prompt_index: 0,
            total_prompts: ENROLLMENT_PROMPTS.len(),
            prompts: ENROLLMENT_PROMPTS.iter().map(|s| s.to_string()).collect(),
            flow: o.flow.flow_name().to_string(),
        },
    );

    true
}

pub fn begin_voice_collection(app: &AppHandle, onboarding: &SharedOnboarding) {
    let mut o = onboarding.lock().unwrap();
    o.stage = OnboardingStage::CollectingVoice { prompt_index: 0 };

    emit_enrollment(
        app,
        EnrollmentEvent {
            stage: o.flow.stage("prompt"),
            message: format!("Read this sentence: \"{}\"", ENROLLMENT_PROMPTS[0]),
            prompt_index: 0,
            total_prompts: ENROLLMENT_PROMPTS.len(),
            prompts: ENROLLMENT_PROMPTS.iter().map(|s| s.to_string()).collect(),
            flow: o.flow.flow_name().to_string(),
        },
    );

    eprintln!("[onboarding] voice collection started — prompt 0");
}

/// Returns true when all prompts are collected.
pub fn record_embedding(
    app: &AppHandle,
    onboarding: &SharedOnboarding,
    embedding: Vec<f32>,
) -> bool {
    let mut o = onboarding.lock().unwrap();

    let OnboardingStage::CollectingVoice { prompt_index } = o.stage else {
        return false;
    };

    o.embeddings.push(embedding);
    let next = prompt_index + 1;

    if next >= ENROLLMENT_PROMPTS.len() {
        o.stage = OnboardingStage::Done;
        eprintln!("[onboarding] all prompts collected");
        return true;
    }

    o.stage = OnboardingStage::CollectingVoice { prompt_index: next };

    emit_enrollment(
        app,
        EnrollmentEvent {
            stage: o.flow.stage("prompt"),
            message: format!("Read this sentence: \"{}\"", ENROLLMENT_PROMPTS[next]),
            prompt_index: next,
            total_prompts: ENROLLMENT_PROMPTS.len(),
            prompts: ENROLLMENT_PROMPTS.iter().map(|s| s.to_string()).collect(),
            flow: o.flow.flow_name().to_string(),
        },
    );

    eprintln!("[onboarding] prompt {next}/{}", ENROLLMENT_PROMPTS.len());
    false
}

pub fn average_embeddings(embeddings: &[Vec<f32>]) -> Vec<f32> {
    if embeddings.is_empty() {
        return vec![];
    }
    let len = embeddings[0].len();
    let mut avg = vec![0.0_f32; len];
    for emb in embeddings {
        for (i, v) in emb.iter().enumerate() {
            avg[i] += v;
        }
    }
    let n = embeddings.len() as f32;
    avg.iter_mut().for_each(|v| *v /= n);
    avg
}
