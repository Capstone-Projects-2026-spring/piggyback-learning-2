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
    pub last_transcript: Option<String>,
    pub mode: SessionMode,
}

impl SessionContext {
    pub fn is_identified(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn set_user(&mut self, id: i32, name: String, role: String) {
        eprintln!("[session] identified user_id={id} name={name:?} role={role:?}");
        self.user_id = Some(id);
        self.user_name = Some(name);
        self.role = Some(role);
    }

    /// Switch to AnswerMode. The next Whisper transcript will be scored
    /// as an answer rather than dispatched as a voice command.
    pub fn enter_answer_mode(&mut self, expected: String, video_id: String, segment_id: i32) {
        eprintln!("[session] → AnswerMode segment={segment_id}");
        self.mode = SessionMode::Answer;
        self.expected_answer = Some(expected);
        self.current_video = Some(video_id);
        self.current_segment = Some(segment_id);
        self.last_transcript = None; // clear stale transcript from previous cycle
    }

    /// Return to CommandMode. Clears answer-specific fields but preserves
    /// current_video and current_segment; those belong to the broader session.
    pub fn exit_answer_mode(&mut self) {
        eprintln!("[session] → CommandMode");
        self.mode = SessionMode::Command;
        self.expected_answer = None;
        self.last_transcript = None;
    }
}

pub type SharedSession = Arc<Mutex<SessionContext>>;

pub fn new_session() -> SharedSession {
    Arc::new(Mutex::new(SessionContext::default()))
}
