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

pub mod crates;
#[cfg(feature = "ai")]
pub mod duplication;
pub mod exports;
pub mod imports;
pub mod macro_scanning;
