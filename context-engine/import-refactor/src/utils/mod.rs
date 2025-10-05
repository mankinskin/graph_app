pub mod file_operations;
pub mod import_analysis;
pub mod import_replacement;
pub mod macro_scanning;
pub mod pub_use_generation;
pub mod usage_analyzer;

// Unified modules for improved code reuse
pub mod common;
pub mod duplication_analyzer;
pub mod refactoring_analyzer;
pub mod analyzer_cli;
