// Code analysis functionality

pub use self::crates::{CrateAnalyzer, CrateNames, CratePaths};
pub use self::imports::{analyze_imports};
pub use self::exports_analyzer::ExportAnalyzer;

// Conditional public interface for AI features
#[cfg(feature = "ai")]
pub use self::duplication::{CodebaseDuplicationAnalyzer, DuplicationAnalysis, AiProvider};

pub mod crates;
pub mod imports;
pub mod exports_analyzer;
pub mod duplication;
pub mod macro_scanning;
mod compilation;
