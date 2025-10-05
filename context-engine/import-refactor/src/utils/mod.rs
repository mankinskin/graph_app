pub mod exports;
pub mod file_operations;
pub mod import_analysis;
pub mod import_replacement;
pub mod pub_use_generation;
pub mod pub_use_merger;

// Unified modules for improved code reuse
pub mod analyzer_cli;
pub mod common;
pub mod duplication_analyzer;
pub mod refactoring_analyzer;

// AI-powered analysis
pub mod ai_client;
