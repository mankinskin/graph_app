// Code analysis functionality

pub use self::crates::{CrateAnalyzer, CrateNames, CratePaths};
pub use self::imports::{ImportAnalysis, analyze_imports, print_analysis_summary};
pub use self::exports::{ExportAnalysis, ExistingExportParser};
pub use self::exports_analyzer::ExportAnalyzer;

// Conditional public interface for AI features
#[cfg(feature = "ai")]
pub use self::duplication::{CodebaseDuplicationAnalyzer, DuplicationAnalysis, AiProvider};

pub mod crates;
pub mod imports;
pub mod exports;
pub mod exports_analyzer;
pub mod duplication;
pub mod macro_scanning;
mod compilation;

// Re-export utility functions from exports - now using navigator
use std::collections::BTreeSet;
use crate::syntax::navigator::{UseTreeNavigator, ItemNameCollector};

/// Recursively extract exported item names from a use tree (using unified navigator)
pub fn extract_exported_items_from_use_tree(
    tree: &syn::UseTree,
    exported_items: &mut BTreeSet<String>,
) {
    let navigator = UseTreeNavigator;
    let mut collector = ItemNameCollector::new();
    navigator.extract_items(tree, &mut collector);
    exported_items.extend(collector.items);
}
