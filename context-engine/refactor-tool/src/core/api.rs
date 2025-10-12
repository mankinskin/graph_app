//! High-level API for running import refactoring operations.
//!
//! This module provides a unified interface for import refactoring that can be used
//! by both the CLI application and external applications.

use anyhow::Result;
use indexmap::IndexSet;
use std::path::Path;

use crate::{
    analysis::crates::{CrateAnalyzer, CrateNames, CratePaths},
    common::format::format_relative_path,
    core::engine::RefactorEngine,
    core::steps::{RefactorStep, RefactorStepsConfig, RefactorStepsManager},
    syntax::parser::{ImportInfo, ImportParser},
};

/// Configuration for a refactoring operation
#[derive(Debug, Clone)]
pub struct RefactorConfig {
    /// The crate names involved in the refactoring
    pub crate_names: CrateNames,
    /// Path to the workspace root
    pub workspace_root: std::path::PathBuf,
    /// Whether to run in dry-run mode (no actual file modifications)
    pub dry_run: bool,
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Whether to suppress progress messages (useful for tests)
    pub quiet: bool,
    /// Whether to keep super:: imports as-is (default: normalize to crate:: format)
    pub keep_super: bool,
    /// Whether to disable automatic export generation (default: false, exports enabled)
    pub keep_exports: bool,
}

/// High-level result of a refactoring operation
#[derive(Debug)]
pub struct RefactorResult {
    /// Whether the refactoring was successful
    pub success: bool,
    /// Number of import statements processed
    pub imports_processed: usize,
    /// Paths of the crates involved
    pub crate_paths: CratePaths,
    /// The steps that were executed
    pub steps_executed: Vec<RefactorStep>,
    /// Whether this was a no-op operation
    pub was_no_op: bool,
    /// Any error that occurred
    pub error: Option<anyhow::Error>,
}

/// High-level API for import refactoring operations
pub struct RefactorApi;

impl RefactorApi {
    /// Execute a complete refactoring operation with the given configuration
    pub fn execute_refactor(config: RefactorConfig) -> RefactorResult {
        match Self::execute_refactor_internal(config) {
            Ok((imports_processed, crate_paths, steps_executed, was_no_op)) => {
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
                crate_paths: CratePaths::SelfCrate {
                    crate_path: std::path::PathBuf::new(),
                },
                steps_executed: Vec::new(),
                was_no_op: false,
                error: Some(error),
            },
        }
    }

    /// Internal implementation of the refactoring logic
    fn execute_refactor_internal(
        config: RefactorConfig
    ) -> Result<(usize, CratePaths, Vec<RefactorStep>, bool)> {
        let RefactorConfig {
            crate_names,
            workspace_root,
            dry_run,
            verbose,
            quiet,
            keep_super,
            keep_exports,
        } = config;

        // Step 0: Determine what refactoring steps are requested
        let steps_config = RefactorStepsConfig::from_flags(
            keep_super,
            keep_exports,
            dry_run,
            crate_names.clone(),
        );

        let steps_manager = RefactorStepsManager::new(steps_config)?;

        // Early exit if no refactoring work is needed
        if !steps_manager.has_work() {
            if !quiet {
                println!("🔧 Import Refactor Tool");
                println!(
                    "ℹ️  No refactoring steps requested - analysis only mode"
                );
                println!();
            }

            // Still need to discover crates for the result
            let analyzer = CrateAnalyzer::new(&workspace_root)?;
            let crate_paths = analyzer.find_crates(&crate_names)?;

            if !quiet {
                crate_paths.print_found(&workspace_root);
                println!("✅ No-op completed - no changes made");
            }

            return Ok((0, crate_paths, Vec::new(), true));
        }

        let summary = steps_manager.summary();

        if !quiet {
            println!("🔧 Import Refactor Tool");
            match &crate_names {
                CrateNames::SelfCrate { crate_name } => {
                    println!(
                        "📦 Crate: {} → will move crate:: imports to root-level exports",
                        crate_name
                    );
                },
                CrateNames::CrossCrate {
                    source_crate,
                    target_crate,
                } => {
                    println!(
                        "📦 Source: {} → Target: {} (will move exports from source to target)",
                        source_crate, target_crate
                    );
                },
            }
            println!("📁 Workspace: {}", format_relative_path(&workspace_root));
            if dry_run {
                println!("🔍 Dry run mode - no files will be modified");
            }
            println!("🎯 Plan: {}", summary.describe());
            println!();
        }

        // Step 1: Discover and validate crates
        let analyzer = CrateAnalyzer::new(&workspace_root)?;
        let crate_paths = analyzer.find_crates(&crate_names)?;

        if !quiet {
            crate_paths.print_found(&workspace_root);
        }

        let mut imports_processed = 0;

        // Step 2: Execute requested steps in dependency order
        if steps_manager.will_execute(RefactorStep::ParseImports)
            || steps_manager.will_execute(RefactorStep::AnalyzeImports)
        {
            // Parse imports based on the type of refactoring
            let imports = match &crate_names {
                CrateNames::SelfCrate { crate_name } => {
                    // For self-refactoring, parse both crate:: imports and explicit crate name imports
                    let source_path = crate_paths.source_path();

                    // Parse crate:: imports
                    let crate_parser = ImportParser::new("crate");
                    let crate_imports =
                        crate_parser.find_imports_in_crate(source_path)?;

                    // Parse explicit crate name imports
                    let explicit_parser = ImportParser::new(crate_name);
                    let explicit_imports =
                        explicit_parser.find_imports_in_crate(source_path)?;

                    // Combine both types of imports using IndexSet for automatic deduplication
                    let mut unique_imports: IndexSet<ImportInfo> =
                        crate_imports.into_iter().collect();

                    for mut import in explicit_imports {
                        // Normalize explicit crate name imports to crate:: format to avoid duplicates
                        import.normalize_to_crate_format(crate_name);

                        // IndexSet automatically handles deduplication based on ImportInfo's Hash/Eq implementation
                        unique_imports.insert(import);
                    }

                    // Parse super:: imports if we'll be normalizing them
                    if steps_manager
                        .will_execute(RefactorStep::NormalizeSuperImports)
                    {
                        let super_imports =
                            ImportParser::find_super_imports_in_crate(
                                source_path,
                            )?;

                        for mut super_import in super_imports {
                            // Normalize super:: imports to crate:: format
                            super_import
                                .normalize_super_imports(source_path)?;

                            // Add to the unique set (will deduplicate automatically)
                            unique_imports.insert(super_import);
                        }
                    }

                    // Convert back to Vec
                    unique_imports.into_iter().collect()
                },
                CrateNames::CrossCrate { source_crate, .. } => {
                    // For cross-crate refactoring, parse target crate imports of source crate
                    let target_path = crate_paths.target_path();
                    let import_parser = ImportParser::new(source_crate);
                    import_parser.find_imports_in_crate(target_path)?
                },
            };

            imports_processed = imports.len();

            if !quiet {
                println!("📊 Analysis Results:");
                println!("   Found {} imports to refactor", imports.len());
                if verbose {
                    for import in &imports {
                        println!(
                            "   📄 {} (line {})",
                            format_relative_path(&import.file_path),
                            import.line_number
                        );
                    }
                }
                println!();
            }

            // Step 3: Execute refactoring if requested
            if steps_manager.will_execute(RefactorStep::GenerateExports)
                || steps_manager.will_execute(RefactorStep::ReplaceImports)
            {
                let mut refactor_engine = RefactorEngine::new(
                    &crate_names,
                    dry_run,
                    verbose,
                    !steps_manager
                        .will_execute(RefactorStep::NormalizeSuperImports), // keep_super
                    !steps_manager.will_execute(RefactorStep::GenerateExports), // keep_exports
                );

                refactor_engine.refactor_imports(
                    &crate_paths,
                    imports,
                    &workspace_root,
                )?;
            }

            // Step 3b: Handle super:: normalization separately if requested
            if steps_manager.will_execute(RefactorStep::NormalizeSuperImports)
                && !steps_manager.will_execute(RefactorStep::GenerateExports)
                && !steps_manager.will_execute(RefactorStep::ReplaceImports)
            {
                // Parse super:: imports WITHOUT normalizing them first
                let source_path = crate_paths.source_path();
                let super_imports =
                    ImportParser::find_super_imports_in_crate(source_path)?;

                if !super_imports.is_empty() {
                    if verbose {
                        println!("🔄 Normalizing {} super:: imports to crate:: format", super_imports.len());
                    }

                    // Create a custom replacement strategy that normalizes during replacement
                    let strategy = crate::syntax::super_strategy::SuperNormalizationStrategy {
                        crate_root: source_path.to_path_buf(),
                    };

                    // Apply the replacement using the super normalization strategy
                    crate::syntax::transformer::replace_imports_with_strategy(
                        super_imports,
                        strategy,
                        &workspace_root,
                        dry_run,
                        verbose,
                    )?;
                }
            }
        }

        if !quiet {
            if dry_run {
                println!("✅ Dry run completed successfully!");
            } else {
                println!("✅ Refactoring completed successfully!");
            }
        }

        Ok((
            imports_processed,
            crate_paths,
            steps_manager.execution_plan().to_vec(),
            summary.is_no_op(),
        ))
    }
}

/// Builder for RefactorConfig to make it easier to construct
pub struct RefactorConfigBuilder {
    crate_names: Option<CrateNames>,
    workspace_root: Option<std::path::PathBuf>,
    dry_run: bool,
    verbose: bool,
    quiet: bool,
    keep_super: bool,
    keep_exports: bool,
}

impl RefactorConfigBuilder {
    pub fn new() -> Self {
        Self {
            crate_names: None,
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
            keep_super: false,
            keep_exports: false,
        }
    }

    pub fn crate_names(
        mut self,
        crate_names: CrateNames,
    ) -> Self {
        self.crate_names = Some(crate_names);
        self
    }

    pub fn workspace_root(
        mut self,
        workspace_root: impl AsRef<Path>,
    ) -> Self {
        self.workspace_root = Some(workspace_root.as_ref().to_path_buf());
        self
    }

    pub fn dry_run(
        mut self,
        dry_run: bool,
    ) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn verbose(
        mut self,
        verbose: bool,
    ) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn quiet(
        mut self,
        quiet: bool,
    ) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn keep_super(
        mut self,
        keep_super: bool,
    ) -> Self {
        self.keep_super = keep_super;
        self
    }

    pub fn keep_exports(
        mut self,
        keep_exports: bool,
    ) -> Self {
        self.keep_exports = keep_exports;
        self
    }

    pub fn build(self) -> Result<RefactorConfig> {
        let crate_names = self
            .crate_names
            .ok_or_else(|| anyhow::anyhow!("crate_names is required"))?;
        let workspace_root = self
            .workspace_root
            .ok_or_else(|| anyhow::anyhow!("workspace_root is required"))?;

        Ok(RefactorConfig {
            crate_names,
            workspace_root,
            dry_run: self.dry_run,
            verbose: self.verbose,
            quiet: self.quiet,
            keep_super: self.keep_super,
            keep_exports: self.keep_exports,
        })
    }
}

impl Default for RefactorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
