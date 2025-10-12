// Code analysis functionality

pub use self::exports::ExportAnalyzer;

pub mod crates;
#[cfg(feature = "ai")]
pub mod duplication;
pub mod exports;
pub mod imports;
pub mod macro_scanning;
