#![feature(iter_intersperse)]

// Import-Refactor Crate - Modular Architecture
//
// This crate provides tools for refactoring Rust import statements and analyzing
// code duplications with optional AI assistance.

// Core refactoring functionality - always available
pub use crate::{
    analysis::{
        analyze_imports,
        CrateAnalyzer,
        CrateNames,
        CratePaths,
    },
    core::{
        RefactorApi,
        RefactorConfig,
        RefactorConfigBuilder,
        RefactorResult,
    },
    syntax::parser::ImportParser,
};

// Feature-gated APIs
#[cfg(feature = "ai")]
pub use crate::ai::{
    AiClient,
    AiClientFactory,
};

#[cfg(feature = "embedded-llm")]
pub use crate::server::{
    CandleServer,
    ServerConfig,
};

// Module declarations
mod analysis;
mod common;
mod core;
mod io;
mod syntax;

// Feature-gated modules
#[cfg(feature = "ai")]
mod ai;

#[cfg(feature = "embedded-llm")]
pub mod server;

// CLI module is private - only used by main.rs
#[cfg(not(test))]
mod cli;
