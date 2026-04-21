use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub role: Option<String>,   // "parent" | "kid"
    pub parent_id: Option<i32>, // set if role == "kid"
    pub current_video: Option<String>,
    pub current_segment: Option<i32>,
    pub expected_answer: Option<String>,
    pub last_audio_path: Option<String>,
}

impl SessionContext {
    pub fn is_identified(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn set_user(&mut self, id: i32, name: String, role: String, parent_id: Option<i32>) {
        self.user_id = Some(id);
        self.user_name = Some(name);
        self.role = Some(role);
        self.parent_id = parent_id;
        eprintln!("[session] identified as user_id={id}");
    }

    pub fn clear_user(&mut self) {
        eprintln!("[session] clearing user — id={:?}", self.user_id);
        self.user_id = None;
        self.user_name = None;
        self.role = None;
        self.parent_id = None;
    }
}

// Global session — Arc<Mutex<>> so capture thread + handlers can both access it
pub type SharedSession = Arc<Mutex<SessionContext>>;

pub fn new_session() -> SharedSession {
    Arc::new(Mutex::new(SessionContext::default()))
}
