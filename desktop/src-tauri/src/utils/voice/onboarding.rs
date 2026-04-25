use super::enrollment::{emit_enrollment, EnrollmentEvent, ENROLLMENT_PROMPTS};
use crate::utils::text::{capitalise_words, clean_transcript, is_valid_name};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum OnboardingFlow {
    Parent,
    Kid,
}

impl OnboardingFlow {
    /// The role string stored in the DB.
    pub fn role(&self) -> &'static str {
        match self {
            OnboardingFlow::Parent => "parent",
            OnboardingFlow::Kid => "kid",
        }
    }

    /// Prefix event stage names for kid flow ("greet" → "kid_greet").
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

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            stage: OnboardingStage::Idle,
            flow: OnboardingFlow::Parent,
            name: None,
            embeddings: Vec::new(),
        }
    }
}

impl OnboardingState {
    pub fn is_active(&self) -> bool {
        !matches!(self.stage, OnboardingStage::Idle | OnboardingStage::Done)
    }
}

pub type SharedOnboarding = Arc<Mutex<OnboardingState>>;

pub fn new_onboarding() -> SharedOnboarding {
    Arc::new(Mutex::new(OnboardingState::default()))
}

pub fn start(onboarding: &SharedOnboarding, flow: OnboardingFlow) {
    let mut o = onboarding.lock().unwrap();
    *o = OnboardingState::default();
    o.flow = flow;
    o.stage = OnboardingStage::WaitingForName;

    let message = match o.flow {
        OnboardingFlow::Parent => {
            "Hi there! I'm Jarvis, your learning buddy. Let's get started — what's your name?"
                .to_string()
        }
        OnboardingFlow::Kid => "Hi! Let's set up a new kid account. What's your name?".to_string(),
    };

    let event = enrollment_event(&o, "greet", message, 0);
    drop(o);

    emit_enrollment(event);
    eprintln!("[onboarding] started — waiting for name");
}

pub fn try_set_name(onboarding: &SharedOnboarding, transcript: &str) -> bool {
    let cleaned = clean_transcript(transcript);
    if !is_valid_name(&cleaned) {
        eprintln!("[onboarding] rejected name: '{transcript}' → '{cleaned}'");
        return false;
    }

    let name = capitalise_words(&cleaned);
    eprintln!("[onboarding] accepted name: '{name}'");

    let mut o = onboarding.lock().unwrap();
    o.name = Some(name.clone());

    let message = match o.flow {
        OnboardingFlow::Parent => format!(
            "Nice to meet you, {name}! Now I need to learn your voice. \
             Read each sentence below out loud clearly."
        ),
        OnboardingFlow::Kid => format!(
            "Great, {name}! Now I need to learn your voice. \
             Read each sentence below out loud clearly."
        ),
    };

    let event = enrollment_event(&o, "name_confirmed", message, 0);
    drop(o);

    emit_enrollment(event);
    true
}

pub fn begin_voice_collection(onboarding: &SharedOnboarding) {
    let mut o = onboarding.lock().unwrap();
    o.stage = OnboardingStage::CollectingVoice { prompt_index: 0 };

    let event = enrollment_event(
        &o,
        "prompt",
        format!("Read this sentence: \"{}\"", ENROLLMENT_PROMPTS[0]),
        0,
    );
    drop(o);

    emit_enrollment(event);
    eprintln!("[onboarding] voice collection started — prompt 0");
}

/// Returns true when all prompts have been collected.
pub fn record_embedding(onboarding: &SharedOnboarding, embedding: Vec<f32>) -> bool {
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

    let event = enrollment_event(
        &o,
        "prompt",
        format!("Read this sentence: \"{}\"", ENROLLMENT_PROMPTS[next]),
        next,
    );
    drop(o);

    emit_enrollment(event);
    eprintln!("[onboarding] prompt {next}/{}", ENROLLMENT_PROMPTS.len());
    false
}

pub fn average_embeddings(embeddings: &[Vec<f32>]) -> Vec<f32> {
    if embeddings.is_empty() {
        return vec![];
    }
    let len = embeddings[0].len();
    let n = embeddings.len() as f32;
    let mut avg = vec![0.0_f32; len];
    for emb in embeddings {
        for (i, v) in emb.iter().enumerate() {
            avg[i] += v;
        }
    }
    avg.iter_mut().for_each(|v| *v /= n);
    avg
}

/// Build an EnrollmentEvent from the current onboarding state.
/// Centralises the repetitive prompt list construction.
fn enrollment_event(
    o: &OnboardingState,
    stage: &str,
    message: String,
    prompt_index: usize,
) -> EnrollmentEvent {
    EnrollmentEvent {
        stage: o.flow.stage(stage),
        message,
        prompt_index,
        total_prompts: ENROLLMENT_PROMPTS.len(),
        prompts: ENROLLMENT_PROMPTS.iter().map(|s| s.to_string()).collect(),
        flow: o.flow.role().to_string(),
    }
}
