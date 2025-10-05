#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
}

impl User {
    pub fn new(id: u64, username: String, email: String) -> Self {
        Self { id, username, email }
    }
}

pub fn create_user(username: &str, email: &str) -> Result<User, String> {
    if username.is_empty() || email.is_empty() {
        return Err("Username and email cannot be empty".to_string());
    }
    
    Ok(User::new(1, username.to_string(), email.to_string()))
}

pub fn find_user_by_email(email: &str) -> Option<User> {
    if email == "test@example.com" {
        Some(User::new(1, "testuser".to_string(), email.to_string()))
    } else {
        None
    }
}

pub fn update_user_profile(user: &mut User, new_email: Option<String>) -> Result<(), String> {
    if let Some(email) = new_email {
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        user.email = email;
    }
    Ok(())
}