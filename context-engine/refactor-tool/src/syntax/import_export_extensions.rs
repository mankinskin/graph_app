//! Extension traits and convenience methods for the unified import/export API
//!
//! This module provides ergonomic interfaces and helper methods that make the
//! unified ImportExportProcessor easier to use in common scenarios.

use crate::{
    analysis::crates::{CrateNames, CratePaths},
    syntax::import_export_processor::{
        ImportExportContext, ImportExportProcessor, ImportTree,
        ProcessingResults,
    },
};
use anyhow::Result;
use std::path::Path;

/// Extension trait for CrateNames to easily create ImportExportContext
pub trait CrateNamesExt {
    /// Create an ImportExportContext with default settings
    fn to_context(
        &self,
        crate_paths: CratePaths,
        workspace_root: &Path,
    ) -> ImportExportContext;

    /// Create an ImportExportContext for dry-run operations
    fn to_dry_run_context(
        &self,
        crate_paths: CratePaths,
        workspace_root: &Path,
    ) -> ImportExportContext;

    /// Create an ImportExportContext with verbose output
    fn to_verbose_context(
        &self,
        crate_paths: CratePaths,
        workspace_root: &Path,
    ) -> ImportExportContext;
}

impl CrateNamesExt for CrateNames {
    fn to_context(
        &self,
        crate_paths: CratePaths,
        workspace_root: &Path,
    ) -> ImportExportContext {
        ImportExportContext::new(
            self.clone(),
            crate_paths,
            workspace_root.to_path_buf(),
        )
    }

    fn to_dry_run_context(
        &self,
        crate_paths: CratePaths,
        workspace_root: &Path,
    ) -> ImportExportContext {
        self.to_context(crate_paths, workspace_root)
            .with_dry_run(true)
    }

    fn to_verbose_context(
        &self,
        crate_paths: CratePaths,
        workspace_root: &Path,
    ) -> ImportExportContext {
        self.to_context(crate_paths, workspace_root)
            .with_verbose(true)
    }
}

/// Extension trait for ImportExportContext to provide builder-style configuration
pub trait ImportExportContextExt {
    /// Configure for cross-crate refactoring with standard settings
    fn for_cross_crate(self) -> Self;

    /// Configure for self-crate refactoring with standard settings  
    fn for_self_crate(self) -> Self;

    /// Configure for super:: import normalization only
    fn for_super_normalization(self) -> Self;

    /// Configure for export generation only
    fn for_export_generation(self) -> Self;
}

impl ImportExportContextExt for ImportExportContext {
    fn for_cross_crate(self) -> Self {
        self.with_normalize_super(false) // Usually not needed for cross-crate
            .with_generate_exports(true) // Generate exports in source crate
    }

    fn for_self_crate(self) -> Self {
        self.with_normalize_super(true) // Normalize super:: imports
            .with_generate_exports(true) // Generate exports for better organization
    }

    fn for_super_normalization(self) -> Self {
        self.with_normalize_super(true).with_generate_exports(false) // Only normalize, don't generate exports
    }

    fn for_export_generation(self) -> Self {
        self.with_normalize_super(false) // Only generate exports, don't normalize
            .with_generate_exports(true)
    }
}

/// Quick-start functions for common use cases
pub struct ImportExportUtils;

impl ImportExportUtils {
    /// Process imports for cross-crate refactoring
    /// This is the most common case where imports from source_crate in target_crate
    /// should be replaced with glob imports, and source_crate should export the items.
    pub fn process_cross_crate(
        source_crate: &str,
        target_crate: &str,
        crate_paths: CratePaths,
        workspace_root: &Path,
        dry_run: bool,
        verbose: bool,
    ) -> Result<ProcessingResults> {
        let crate_names = CrateNames::CrossCrate {
            source_crate: source_crate.to_string(),
            target_crate: target_crate.to_string(),
        };

        let context = crate_names
            .to_context(crate_paths, workspace_root)
            .with_dry_run(dry_run)
            .with_verbose(verbose)
            .for_cross_crate();

        let processor = ImportExportProcessor::new(context);
        processor.process()
    }

    /// Process imports for self-crate refactoring
    /// This normalizes crate:: imports within a single crate to use root-level exports.
    pub fn process_self_crate(
        crate_name: &str,
        crate_path: &Path,
        workspace_root: &Path,
        dry_run: bool,
        verbose: bool,
    ) -> Result<ProcessingResults> {
        let crate_names = CrateNames::SelfCrate {
            crate_name: crate_name.to_string(),
        };

        let crate_paths = CratePaths::SelfCrate {
            crate_path: crate_path.to_path_buf(),
        };

        let context = crate_names
            .to_context(crate_paths, workspace_root)
            .with_dry_run(dry_run)
            .with_verbose(verbose)
            .for_self_crate();

        let processor = ImportExportProcessor::new(context);
        processor.process()
    }

    /// Normalize super:: imports to crate:: format only
    /// This is useful when you want to clean up super:: imports without other changes.
    pub fn normalize_super_imports(
        crate_name: &str,
        crate_path: &Path,
        workspace_root: &Path,
        dry_run: bool,
        verbose: bool,
    ) -> Result<ProcessingResults> {
        let crate_names = CrateNames::SelfCrate {
            crate_name: crate_name.to_string(),
        };

        let crate_paths = CratePaths::SelfCrate {
            crate_path: crate_path.to_path_buf(),
        };

        let context = crate_names
            .to_context(crate_paths, workspace_root)
            .with_dry_run(dry_run)
            .with_verbose(verbose)
            .for_super_normalization();

        let processor = ImportExportProcessor::new(context);
        processor.process()
    }

    /// Generate exports for existing imports without changing the imports themselves
    /// This is useful when you want to create a clean public API.
    pub fn generate_exports_only(
        crate_name: &str,
        crate_path: &Path,
        workspace_root: &Path,
        dry_run: bool,
        verbose: bool,
    ) -> Result<ProcessingResults> {
        let crate_names = CrateNames::SelfCrate {
            crate_name: crate_name.to_string(),
        };

        let crate_paths = CratePaths::SelfCrate {
            crate_path: crate_path.to_path_buf(),
        };

        let context = crate_names
            .to_context(crate_paths, workspace_root)
            .with_dry_run(dry_run)
            .with_verbose(verbose)
            .for_export_generation();

        let processor = ImportExportProcessor::new(context);
        processor.process()
    }
}

/// Extension trait for ProcessingResults to provide convenience methods
pub trait ProcessingResultsExt {
    /// Print a summary of what was processed
    fn print_summary(
        &self,
        verbose: bool,
    );

    /// Check if any changes were made
    fn has_changes(&self) -> bool;

    /// Get a human-readable description of the changes
    fn describe_changes(&self) -> Vec<String>;
}

impl ProcessingResultsExt for ProcessingResults {
    fn print_summary(
        &self,
        verbose: bool,
    ) {
        println!("📊 Processing Summary:");
        println!("   • Total imports processed: {}", self.total_imports());
        println!("   • Replacements made: {}", self.total_replacements());
        println!("   • Exports generated: {}", self.total_exports_generated());
        println!(
            "   • Normalizations applied: {}",
            self.normalization_changes
        );

        if verbose {
            let changes = self.describe_changes();
            if !changes.is_empty() {
                println!("\n📝 Detailed Changes:");
                for change in changes {
                    println!("   {}", change);
                }
            }
        }
    }

    fn has_changes(&self) -> bool {
        self.total_replacements() > 0
            || self.total_exports_generated() > 0
            || self.normalization_changes > 0
    }

    fn describe_changes(&self) -> Vec<String> {
        let mut changes = Vec::new();

        // Describe replacement changes
        for (file_path, actions) in &self.replacement_results {
            let file_name = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");

            for action in actions {
                match action {
                    crate::syntax::transformer::ReplacementAction::Replaced { from, to } => {
                        changes.push(format!("🔄 {}: {} → {}", file_name, from, to));
                    },
                    crate::syntax::transformer::ReplacementAction::Removed { original } => {
                        changes.push(format!("❌ {}: Removed {}", file_name, original));
                    },
                    crate::syntax::transformer::ReplacementAction::NotFound { searched_for } => {
                        changes.push(format!("⚠️  {}: Could not find {}", file_name, searched_for));
                    },
                }
            }
        }

        // Describe super import normalization
        if let Some(super_results) = &self.super_results {
            for (file_path, actions) in super_results {
                let file_name = file_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");

                for action in actions {
                    if let crate::syntax::transformer::ReplacementAction::Replaced { from, to } = action {
                        changes.push(format!("🎯 {}: Normalized {} → {}", file_name, from, to));
                    }
                }
            }
        }

        // Describe export generation
        if let Some(export_analysis) = &self.export_analysis {
            if !export_analysis.generated_statements.is_empty() {
                changes.push(format!(
                    "📤 Generated {} export statements in lib.rs",
                    export_analysis.generated_statements.len()
                ));

                for statement in &export_analysis.generated_statements {
                    changes.push(format!("   + {}", statement.trim()));
                }
            }
        }

        changes
    }
}

/// Extension trait for ImportTree to provide analysis methods
pub trait ImportTreeExt {
    /// Get statistics about the import tree
    fn stats(&self) -> ImportTreeStats;

    /// Find potential optimizations (like groupable imports)
    fn find_optimizations(&self) -> Vec<ImportOptimization>;
}

#[derive(Debug)]
pub struct ImportTreeStats {
    pub simple_imports: usize,
    pub grouped_imports: usize,
    pub super_imports: usize,
    pub total_items: usize,
    pub unique_modules: usize,
}

#[derive(Debug)]
pub enum ImportOptimization {
    /// Multiple imports from the same module that could be grouped
    GroupableImports {
        module_path: String,
        import_count: usize,
        potential_grouping: String,
    },
    /// Redundant imports that import the same item multiple times
    RedundantImports {
        item_path: String,
        occurrence_count: usize,
    },
    /// Super imports that should be normalized
    UnnormalizedSuperImports { count: usize },
}

impl ImportTreeExt for ImportTree {
    fn stats(&self) -> ImportTreeStats {
        let grouped_item_count: usize =
            self.grouped_imports.values().map(|items| items.len()).sum();

        let total_items = self.simple_imports.len()
            + grouped_item_count
            + self.super_imports.len();

        // Count unique modules
        let mut modules = std::collections::HashSet::new();

        for import in &self.simple_imports {
            if let Some(module) = import.import_path.rsplitn(2, "::").nth(1) {
                modules.insert(module.to_string());
            }
        }

        for module_path in self.grouped_imports.keys() {
            modules.insert(module_path.clone());
        }

        for import in &self.super_imports {
            if let Some(module) = import.import_path.rsplitn(2, "::").nth(1) {
                modules.insert(module.to_string());
            }
        }

        ImportTreeStats {
            simple_imports: self.simple_imports.len(),
            grouped_imports: self.grouped_imports.len(),
            super_imports: self.super_imports.len(),
            total_items,
            unique_modules: modules.len(),
        }
    }

    fn find_optimizations(&self) -> Vec<ImportOptimization> {
        let mut optimizations = Vec::new();

        // Find groupable imports
        let mut module_counts: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for import in &self.simple_imports {
            let parts: Vec<&str> =
                import.import_path.rsplitn(2, "::").collect();
            if let [item, module_path] = parts.as_slice() {
                module_counts
                    .entry(module_path.to_string())
                    .or_default()
                    .push(item.to_string());
            }
        }

        for (module_path, items) in module_counts {
            if items.len() > 1 {
                let potential_grouping =
                    format!("use {}::{{{}}};", module_path, items.join(", "));
                optimizations.push(ImportOptimization::GroupableImports {
                    module_path,
                    import_count: items.len(),
                    potential_grouping,
                });
            }
        }

        // Find unnormalized super imports
        if !self.super_imports.is_empty() {
            optimizations.push(ImportOptimization::UnnormalizedSuperImports {
                count: self.super_imports.len(),
            });
        }

        optimizations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_crate_names_ext() {
        let crate_names = CrateNames::SelfCrate {
            crate_name: "test_crate".to_string(),
        };

        let crate_paths = CratePaths::SelfCrate {
            crate_path: PathBuf::from("/test"),
        };

        let context = crate_names
            .to_verbose_context(crate_paths, Path::new("/workspace"));

        assert!(context.verbose);
        assert!(!context.dry_run);
        assert_eq!(context.workspace_root, PathBuf::from("/workspace"));
    }

    #[test]
    fn test_import_tree_stats() {
        let mut tree = ImportTree::new();

        tree.simple_imports.push(crate::syntax::parser::ImportInfo {
            file_path: PathBuf::from("test.rs"),
            import_path: "std::collections::HashMap".to_string(),
            line_number: 1,
            imported_items: vec!["HashMap".to_string()],
        });

        tree.grouped_imports.insert(
            "std::collections".to_string(),
            vec!["HashSet".to_string(), "BTreeMap".to_string()],
        );

        let stats = tree.stats();

        assert_eq!(stats.simple_imports, 1);
        assert_eq!(stats.grouped_imports, 1);
        assert_eq!(stats.super_imports, 0);
        assert_eq!(stats.total_items, 3); // 1 simple + 2 grouped
    }
}
