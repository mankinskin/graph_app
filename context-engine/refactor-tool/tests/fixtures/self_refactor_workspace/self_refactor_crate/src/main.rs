// This file demonstrates the problem: lots of long imports that could be simplified
// These imports work, but after refactoring they should become simple crate root imports

use self_refactor_crate::core::config::{Config, load_settings, save_settings};
use self_refactor_crate::core::validation::{validate_user_input, ValidationResult, validate_email};
use self_refactor_crate::services::auth::user::{User, create_user, find_user_by_email, update_user_profile};
use self_refactor_crate::services::auth::session::{Session, SessionManager, validate_session};
use self_refactor_crate::services::data::repository::{Repository, InMemoryRepository, create_user_repository, backup_data};
use self_refactor_crate::hello;

fn main() {
    println!("🚀 Starting application...");
    
    // Use config functionality
    let mut config = load_settings();
    config = config.with_debug();
    println!("Config loaded: {:?}", config);
    
    // Validate user input
    let validation = validate_user_input("test@example.com", "password123", "testuser");
    if validation.valid {
        println!("✅ User input is valid");
    } else {
        println!("❌ Validation errors: {:?}", validation.errors);
    }
    
    // Create and manage users
    match create_user("johndoe", "john@example.com") {
        Ok(mut user) => {
            println!("Created user: {:?}", user);
            
            // Update user profile
            if let Err(e) = update_user_profile(&mut user, Some("newemail@example.com".to_string())) {
                println!("Failed to update user: {}", e);
            }
            
            // Find user by email
            if let Some(found_user) = find_user_by_email("test@example.com") {
                println!("Found existing user: {:?}", found_user);
            }
        }
        Err(e) => println!("Failed to create user: {}", e),
    }
    
    // Session management
    let mut session_manager = SessionManager::new();
    let session = session_manager.create_session(1);
    println!("Created session: {:?}", session);
    
    if validate_session(&session) {
        println!("✅ Session is valid");
    }
    
    // Data repository operations
    let mut repo = create_user_repository();
    if let Err(e) = repo.save(1, "testuser".to_string()) {
        println!("Failed to save to repository: {}", e);
    }
    
    let backup_count = backup_data(&repo);
    println!("Backup contains {} items", backup_count);
    
    // Use simple crate root function
    println!("{}", hello());
    
    // Save final config
    if let Err(e) = save_settings(&config) {
        println!("Failed to save settings: {}", e);
    }
    
    println!("✨ Application finished successfully!");
}