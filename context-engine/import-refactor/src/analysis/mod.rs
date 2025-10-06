// Code analysis functionality

#[allow(unused_imports)] // These are re-exported for public API and used by tests
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
pub mod duplication;
pub mod exports;
pub mod imports;
pub mod macro_scanning;
