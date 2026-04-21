use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use super::enrollment::{emit_enrollment, EnrollmentEvent, ENROLLMENT_PROMPTS};

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
    pub parent_name: Option<String>,
    pub embeddings: Vec<Vec<f32>>, // one per prompt, averaged at the end
}

impl OnboardingState {
    pub fn new() -> Self {
        Self {
            stage: OnboardingStage::Idle,
            parent_name: None,
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

/// Called from lib.rs on first run — kicks off the flow
pub fn start(app: &AppHandle, onboarding: &SharedOnboarding) {
    {
        let mut o = onboarding.lock().unwrap();
        o.stage = OnboardingStage::WaitingForName;
    }

    emit_enrollment(
        app,
        EnrollmentEvent {
            stage: "greet".to_string(),
            message: "Hi there! I'm Peppa, your learning buddy. \
                        I'd love to get to know you. \
                        What's your name?"
                .to_string(),
            prompt_index: 0,
            total_prompts: ENROLLMENT_PROMPTS.len(),
        },
    );

    eprintln!("[onboarding] started — waiting for name");
}

/// Call this after we have the parent's name, moves to voice collection
pub fn begin_voice_collection(app: &AppHandle, onboarding: &SharedOnboarding) {
    {
        let mut o = onboarding.lock().unwrap();
        o.stage = OnboardingStage::CollectingVoice { prompt_index: 0 };
    }

    emit_enrollment(
        app,
        EnrollmentEvent {
            stage: "prompt".to_string(),
            message: format!(
                "Great! Now I need to learn your voice. \
             Please read the following sentence out loud:\n\n\"{}\"",
                ENROLLMENT_PROMPTS[0]
            ),
            prompt_index: 0,
            total_prompts: ENROLLMENT_PROMPTS.len(),
        },
    );

    eprintln!("[onboarding] voice collection started — prompt 0");
}

/// Call this each time we get an embedding during collection
/// Returns true when all prompts are done
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
            stage: "prompt".to_string(),
            message: format!(
                "Perfect! Now read this one:\n\n\"{}\"",
                ENROLLMENT_PROMPTS[next]
            ),
            prompt_index: next,
            total_prompts: ENROLLMENT_PROMPTS.len(),
        },
    );

    eprintln!("[onboarding] prompt {next}/{}", ENROLLMENT_PROMPTS.len());
    false
}

/// Average all collected embeddings into one representative vector
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
