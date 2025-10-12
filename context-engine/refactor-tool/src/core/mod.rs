// Core refactoring functionality and abstractions
#[allow(unused_imports)]
// These are re-exported for public API and used by tests
pub use self::api::{
    RefactorApi, RefactorConfig, RefactorConfigBuilder, RefactorResult,
};
pub use self::steps::{
    RefactorStep, RefactorStepsConfig, RefactorStepsManager, RefactorSummary,
};

pub mod api;
pub mod ast_manager;
mod engine;
pub mod path;
pub mod steps;
pub mod unified_adapter;
