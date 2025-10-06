// Core refactoring functionality
pub use self::api::{RefactorApi, RefactorConfig, RefactorResult, RefactorConfigBuilder};

// Re-export CrateNames for convenience (it will be moved to analysis module)
pub use crate::analysis::crates::CrateNames;

pub mod api;
mod engine;