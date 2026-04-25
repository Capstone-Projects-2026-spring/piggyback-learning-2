use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SessionMode {
    #[default]
    Command,
    Answer,
}

#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub role: Option<String>,
    pub current_video: Option<String>,
    pub current_segment: Option<i32>,
    pub expected_answer: Option<String>,
    pub last_audio_path: Option<String>,
    pub last_transcript: Option<String>,
    pub mode: SessionMode,
}

impl SessionContext {
    pub fn is_identified(&self) -> bool {
        self.user_id.is_some()
    }
    pub fn set_user(&mut self, id: i32, name: String, role: String) {
        self.user_id = Some(id);
        self.user_name = Some(name);
        self.role = Some(role);
        eprintln!("[session] identified as user_id={id}");
    }
    pub fn clear_user(&mut self) {
        eprintln!("[session] clearing user — id={:?}", self.user_id);
        self.user_id = None;
        self.user_name = None;
        self.role = None;
    }
    pub fn enter_answer_mode(&mut self, expected: String, video_id: String, segment_id: i32) {
        self.mode = SessionMode::Answer;
        self.expected_answer = Some(expected);
        self.current_video = Some(video_id);
        self.current_segment = Some(segment_id);
        eprintln!("[session] → AnswerMode segment={segment_id}");
    }
    pub fn exit_answer_mode(&mut self) {
        self.mode = SessionMode::Command;
        self.expected_answer = None;
        eprintln!("[session] → CommandMode");
    }
}

pub type SharedSession = Arc<Mutex<SessionContext>>;

pub fn new_session() -> SharedSession {
    Arc::new(Mutex::new(SessionContext::default()))
}
