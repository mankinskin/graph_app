pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 8
}

pub fn validate_username(username: &str) -> bool {
    username.len() >= 3 && username.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

pub fn validate_user_input(email: &str, password: &str, username: &str) -> ValidationResult {
    let mut errors = Vec::new();
    
    if !validate_email(email) {
        errors.push("Invalid email format".to_string());
    }
    if !validate_password(password) {
        errors.push("Password must be at least 8 characters".to_string());
    }
    if !validate_username(username) {
        errors.push("Username must be at least 3 alphanumeric characters".to_string());
    }
    
    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}