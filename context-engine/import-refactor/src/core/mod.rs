// Core refactoring functionality and abstractions
pub use self::api::{RefactorApi, RefactorConfig, RefactorResult, RefactorConfigBuilder};
pub use self::path::ImportPath;
pub use self::ast_manager::{AstManager, CacheStats};

// Re-export CrateNames for convenience (it will be moved to analysis module)
pub use crate::analysis::crates::CrateNames;

pub mod api;
pub mod path;
pub mod ast_manager;
mod engine;