use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: u64,
    pub expires_at: u64,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
    
    pub fn create_session(&mut self, user_id: u64) -> Session {
        let session = Session {
            id: format!("session_{}", user_id),
            user_id,
            expires_at: 3600, // 1 hour
        };
        self.sessions.insert(session.id.clone(), session.clone());
        session
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }
    
    pub fn remove_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }
}

pub fn validate_session(session: &Session) -> bool {
    session.expires_at > 0
}