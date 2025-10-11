// This module demonstrates the problem: lots of long crate:: imports
// These imports work, but they're verbose and could be simplified by re-exporting at crate root

use crate::{
    core::{
        config::{
            load_settings,
            save_settings,
            Config,
        },
        validation::{
            validate_email,
            validate_user_input,
            ValidationResult,
        },
    },
    hello,
    services::{
        auth::{
            session::{
                validate_session,
                Session,
                SessionManager,
            },
            user::{
                create_user,
                find_user_by_email,
                update_user_profile,
                User,
            },
        },
        data::repository::{
            backup_data,
            create_user_repository,
            InMemoryRepository,
            Repository,
        },
    },
};

pub struct Application {
    config: Config,
    session_manager: SessionManager,
    user_repo: InMemoryRepository<User>,
}

impl Application {
    pub fn new() -> Self {
        let config = load_settings();
        let session_manager = SessionManager::new();
        let user_repo = create_user_repository();

        Self {
            config,
            session_manager,
            user_repo,
        }
    }

    pub fn register_user(
        &mut self,
        email: &str,
        password: &str,
        username: &str,
    ) -> Result<User, String> {
        // Validate input using imported validation functions
        let validation = validate_user_input(email, password, username);
        if !validation.valid {
            return Err(format!("Validation failed: {:?}", validation.errors));
        }

        // Create user using imported user functions
        let user = create_user(username, email)?;

        // Store in repository using imported repository traits
        self.user_repo.save(user.id, user.clone())?;

        Ok(user)
    }

    pub fn login_user(
        &mut self,
        email: &str,
    ) -> Result<Session, String> {
        // Find user using imported function
        let user = find_user_by_email(email)
            .ok_or_else(|| "User not found".to_string())?;

        // Create session using imported session manager
        let session = self.session_manager.create_session(user.id);

        // Validate session using imported function
        if !validate_session(&session) {
            return Err("Invalid session".to_string());
        }

        Ok(session)
    }

    pub fn update_user_email(
        &mut self,
        user_id: u64,
        new_email: &str,
    ) -> Result<(), String> {
        // Validate email using imported function
        if !validate_email(new_email) {
            return Err("Invalid email format".to_string());
        }

        // Get user from repository
        let user = self
            .user_repo
            .find(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        // Update user using imported function
        let mut user_copy = user.clone();
        update_user_profile(&mut user_copy, Some(new_email.to_string()))?;

        // Save back to repository
        self.user_repo.save(user_id, user_copy)?;

        Ok(())
    }

    pub fn backup_system(&self) -> usize {
        // Use imported backup function
        backup_data(&self.user_repo)
    }

    pub fn get_greeting(&self) -> String {
        // Use imported hello function
        hello()
    }

    pub fn save_current_config(&self) -> Result<(), String> {
        // Use imported save function
        save_settings(&self.config)
    }
}
