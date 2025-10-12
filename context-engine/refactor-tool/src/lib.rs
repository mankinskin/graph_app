#![feature(iter_intersperse)]
// Allow unused code during testing and development
#![cfg_attr(any(test, debug_assertions), allow(unused))]
#![cfg_attr(any(test, debug_assertions), allow(dead_code))]

// Refactor-Tool Crate - Modular Architecture
//
// This crate provides tools for refactoring Rust import statements and analyzing
// code duplications with optional AI assistance.

// Core refactoring functionality - always available
pub use crate::{
    analysis::{
        crates::{CrateAnalyzer, CrateNames, CratePaths},
        imports::analyze_imports,
    },
    core::{
        path::is_super_import, RefactorApi, RefactorConfig,
        RefactorConfigBuilder, RefactorResult,
    },
    syntax::parser::ImportParser,
};

// Feature-gated APIs
#[cfg(feature = "ai")]
pub use crate::ai::{AiClient, AiClientFactory};

#[cfg(feature = "embedded-llm")]
pub use crate::server::{CandleServer, ServerConfig};

// Module declarations
mod analysis;
mod common;
mod io;

#[cfg(test)]
pub mod core;
#[cfg(test)]
pub mod syntax;

#[cfg(not(test))]
mod core;
#[cfg(not(test))]
mod syntax;

// Feature-gated modules
#[cfg(feature = "ai")]
mod ai;

#[cfg(feature = "embedded-llm")]
pub mod server;

// CLI module is private - only used by main.rs
#[cfg(not(test))]
mod cli;
