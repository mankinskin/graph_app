// Code analysis functionality

pub use self::{
    crates::{
        CrateAnalyzer,
        CrateNames,
        CratePaths,
    },
    exports::ExportAnalyzer,
    imports::analyze_imports,
};

// Conditional public interface for AI features
#[cfg(feature = "ai")]
pub use self::duplication::{
    AiProvider,
    CodebaseDuplicationAnalyzer,
    DuplicationAnalysis,
};

mod compilation;
pub mod crates;
pub mod duplication;
pub mod exports;
pub mod imports;
pub mod macro_scanning;
