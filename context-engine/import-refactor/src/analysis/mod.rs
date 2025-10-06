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

// Re-export utility functions from exports
use std::collections::BTreeSet;

/// Recursively extract exported item names from a use tree
pub fn extract_exported_items_from_use_tree(
    tree: &syn::UseTree,
    exported_items: &mut BTreeSet<String>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            extract_exported_items_from_use_tree(&path.tree, exported_items);
        },
        syn::UseTree::Name(name) => {
            exported_items.insert(name.ident.to_string());
        },
        syn::UseTree::Rename(rename) => {
            exported_items.insert(rename.rename.to_string());
        },
        syn::UseTree::Glob(_) => {
            exported_items.insert("*".to_string());
        },
        syn::UseTree::Group(group) =>
            for item in &group.items {
                extract_exported_items_from_use_tree(item, exported_items);
            },
    }
}
