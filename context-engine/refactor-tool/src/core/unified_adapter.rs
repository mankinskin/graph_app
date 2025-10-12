//! Integration adapter for the unified ImportExportProcessor with the existing RefactorApi
//!
//! This module provides compatibility between the new unified API and the existing
//! refactoring system, allowing for gradual migration and testing.

use crate::{
    core::api::{RefactorConfig, RefactorResult},
    syntax::{
        import_export_extensions::{
            CrateNamesExt, ImportExportContextExt, ProcessingResultsExt,
        },
        import_export_processor::{ImportExportContext, ImportExportProcessor},
    },
};
use anyhow::Result;

/// Adapter that integrates the unified ImportExportProcessor with RefactorApi
pub struct UnifiedApiAdapter;

impl UnifiedApiAdapter {
    /// Execute refactoring using the unified ImportExportProcessor
    ///
    /// This method provides a bridge between RefactorConfig and the new unified API,
    /// allowing existing code to benefit from the improved processing pipeline.
    pub fn execute_with_unified_processor(
        config: RefactorConfig
    ) -> RefactorResult {
        let verbose = config.verbose;
        let quiet = config.quiet;

        match Self::execute_unified_internal(config) {
            Ok((processing_results, crate_paths)) => {
                let imports_processed = processing_results.total_imports();
                let steps_executed = vec![]; // TODO: Map from processing results
                let was_no_op = !processing_results.has_changes();

                if verbose && !quiet {
                    processing_results.print_summary(true);
                }

                RefactorResult {
                    success: true,
                    imports_processed,
                    crate_paths,
                    steps_executed,
                    was_no_op,
                    error: None,
                }
            },
            Err(error) => RefactorResult {
                success: false,
                imports_processed: 0,
                crate_paths: crate::analysis::crates::CratePaths::SelfCrate {
                    crate_path: std::path::PathBuf::new(),
                },
                steps_executed: vec![],
                was_no_op: false,
                error: Some(error),
            },
        }
    }

    /// Internal implementation using the unified processor
    fn execute_unified_internal(
        config: RefactorConfig
    ) -> Result<(
        crate::syntax::import_export_processor::ProcessingResults,
        crate::analysis::crates::CratePaths,
    )> {
        // Analyze crates to get paths
        let analyzer = crate::analysis::crates::CrateAnalyzer::new(
            &config.workspace_root,
        )?;
        let crate_paths = analyzer.find_crates(&config.crate_names)?;

        // Create ImportExportContext from RefactorConfig
        let context = config
            .crate_names
            .to_context(crate_paths.clone(), &config.workspace_root)
            .with_dry_run(config.dry_run)
            .with_verbose(config.verbose)
            .with_normalize_super(!config.keep_super)
            .with_generate_exports(!config.keep_exports);

        // Configure context based on crate type
        let context = match &config.crate_names {
            crate::analysis::crates::CrateNames::CrossCrate { .. } => {
                context.for_cross_crate()
            },
            crate::analysis::crates::CrateNames::SelfCrate { .. } => {
                context.for_self_crate()
            },
        };

        // Execute unified processing
        let processor = ImportExportProcessor::new(context);
        let results = processor.process()?;

        Ok((results, crate_paths))
    }

    /// Compare results between old and new processing methods
    ///
    /// This is useful for testing and validation during migration
    pub fn compare_processing_methods(
        config: RefactorConfig
    ) -> ComparisonResult {
        let unified_result =
            Self::execute_with_unified_processor(config.clone());
        let original_result =
            crate::core::api::RefactorApi::execute_refactor(config);

        ComparisonResult {
            unified_result,
            original_result,
        }
    }
}

/// Results of comparing old vs new processing methods
#[derive(Debug)]
pub struct ComparisonResult {
    pub unified_result: RefactorResult,
    pub original_result: RefactorResult,
}

impl ComparisonResult {
    /// Check if both methods produced equivalent results
    pub fn are_equivalent(&self) -> bool {
        self.unified_result.success == self.original_result.success
            && self.unified_result.imports_processed
                == self.original_result.imports_processed
            && self.unified_result.was_no_op == self.original_result.was_no_op
    }

    /// Get a summary of differences between the methods
    pub fn differences(&self) -> Vec<String> {
        let mut diffs = Vec::new();

        if self.unified_result.success != self.original_result.success {
            diffs.push(format!(
                "Success status differs: unified={}, original={}",
                self.unified_result.success, self.original_result.success
            ));
        }

        if self.unified_result.imports_processed
            != self.original_result.imports_processed
        {
            diffs.push(format!(
                "Import counts differ: unified={}, original={}",
                self.unified_result.imports_processed,
                self.original_result.imports_processed
            ));
        }

        if self.unified_result.was_no_op != self.original_result.was_no_op {
            diffs.push(format!(
                "No-op status differs: unified={}, original={}",
                self.unified_result.was_no_op, self.original_result.was_no_op
            ));
        }

        match (&self.unified_result.error, &self.original_result.error) {
            (Some(_), None) => diffs
                .push("Unified method had error, original didn't".to_string()),
            (None, Some(_)) => diffs
                .push("Original method had error, unified didn't".to_string()),
            (Some(u_err), Some(o_err)) => {
                if u_err.to_string() != o_err.to_string() {
                    diffs.push(
                        "Different error messages between methods".to_string(),
                    );
                }
            },
            (None, None) => {}, // Both successful
        }

        diffs
    }

    /// Print a comparison report
    pub fn print_comparison(&self) {
        println!("🔍 Processing Method Comparison:");
        println!(
            "  Unified API:  success={}, imports={}, no_op={}",
            self.unified_result.success,
            self.unified_result.imports_processed,
            self.unified_result.was_no_op
        );
        println!(
            "  Original API: success={}, imports={}, no_op={}",
            self.original_result.success,
            self.original_result.imports_processed,
            self.original_result.was_no_op
        );

        if self.are_equivalent() {
            println!("  ✅ Results are equivalent");
        } else {
            println!("  ❌ Results differ:");
            for diff in self.differences() {
                println!("    • {}", diff);
            }
        }
    }
}

/// Extension to RefactorConfig for unified API compatibility
pub trait RefactorConfigExt {
    /// Convert RefactorConfig to ImportExportContext
    fn to_import_export_context(
        &self,
        crate_paths: crate::analysis::crates::CratePaths,
    ) -> ImportExportContext;
}

impl RefactorConfigExt for RefactorConfig {
    fn to_import_export_context(
        &self,
        crate_paths: crate::analysis::crates::CratePaths,
    ) -> ImportExportContext {
        ImportExportContext::new(
            self.crate_names.clone(),
            crate_paths,
            self.workspace_root.clone(),
        )
        .with_dry_run(self.dry_run)
        .with_verbose(self.verbose)
        .with_normalize_super(!self.keep_super)
        .with_generate_exports(!self.keep_exports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_refactor_config_conversion() {
        let config = RefactorConfig {
            crate_names: crate::analysis::crates::CrateNames::SelfCrate {
                crate_name: "test_crate".to_string(),
            },
            workspace_root: PathBuf::from("/workspace"),
            dry_run: true,
            verbose: true,
            quiet: false,
            keep_super: false,
            keep_exports: false,
        };

        let crate_paths = crate::analysis::crates::CratePaths::SelfCrate {
            crate_path: PathBuf::from("/workspace/test_crate"),
        };

        let context = config.to_import_export_context(crate_paths);

        assert!(context.dry_run);
        assert!(context.verbose);
        assert!(context.normalize_super); // !keep_super
        assert!(context.generate_exports); // !keep_exports
    }
}
