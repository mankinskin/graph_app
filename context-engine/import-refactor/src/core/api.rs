//! High-level API for running import refactoring operations.
//!
//! This module provides a unified interface for import refactoring that can be used
//! by both the CLI application and external applications.

use anyhow::Result;
use std::path::Path;

use crate::{
    analysis::crates::{
        CrateAnalyzer,
        CrateNames,
        CratePaths,
    },
    common::path::format_relative_path,
    core::engine::RefactorEngine,
    syntax::parser::ImportParser,
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
    /// Any error that occurred
    pub error: Option<anyhow::Error>,
}

/// High-level API for import refactoring operations
pub struct RefactorApi;

impl RefactorApi {
    /// Execute a complete refactoring operation with the given configuration
    pub fn execute_refactor(config: RefactorConfig) -> RefactorResult {
        match Self::execute_refactor_internal(config) {
            Ok((imports_processed, crate_paths)) => RefactorResult {
                success: true,
                imports_processed,
                crate_paths,
                error: None,
            },
            Err(error) => RefactorResult {
                success: false,
                imports_processed: 0,
                crate_paths: match &error.downcast_ref::<anyhow::Error>() {
                    Some(_) => CratePaths::SelfRefactor {
                        crate_path: std::path::PathBuf::new(),
                    },
                    None => CratePaths::SelfRefactor {
                        crate_path: std::path::PathBuf::new(),
                    },
                },
                error: Some(error),
            },
        }
    }

    /// Internal implementation of the refactoring logic
    fn execute_refactor_internal(
        config: RefactorConfig
    ) -> Result<(usize, CratePaths)> {
        let RefactorConfig {
            crate_names,
            workspace_root,
            dry_run,
            verbose,
            quiet,
        } = config;

        if !quiet {
            println!("🔧 Import Refactor Tool");
            match &crate_names {
                CrateNames::SelfRefactor { crate_name } => {
                    println!(
                        "📦 Crate: {} → will move crate:: imports to root-level exports",
                        crate_name
                    );
                },
                CrateNames::CrossRefactor {
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
            println!();
        }

        // Step 1: Discover and validate crates
        let analyzer = CrateAnalyzer::new(&workspace_root)?;
        let crate_paths = analyzer.find_crates(&crate_names)?;

        if !quiet {
            crate_paths.print_found(&workspace_root);
        }

        // Step 2: Parse imports and run refactoring engine
        let mut refactor_engine =
            RefactorEngine::new(&crate_names, dry_run, verbose);

        // Parse imports based on the type of refactoring
        match &crate_names {
            CrateNames::SelfRefactor { .. } => {
                // For self-refactoring, parse crate:: imports in the target crate
                let source_path = crate_paths.source_path();
                let crate_name = match &crate_names {
                    CrateNames::SelfRefactor { crate_name } => crate_name,
                    _ => unreachable!(),
                };
                let import_parser = ImportParser::new(crate_name);
                let imports =
                    import_parser.find_imports_in_crate(source_path)?;

                if !quiet {
                    println!("📊 Analysis Results:");
                    println!(
                        "   Found {} crate:: imports to refactor",
                        imports.len()
                    );
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

                refactor_engine.refactor_imports(
                    &crate_paths,
                    imports,
                    &workspace_root,
                )?;
            },
            CrateNames::CrossRefactor { source_crate, .. } => {
                // For cross-crate refactoring, parse target crate imports of source crate
                let source_path = crate_paths.source_path();
                let target_path = crate_paths.target_path();
                let import_parser = ImportParser::new(source_crate);
                let imports =
                    import_parser.find_imports_in_crate(target_path)?;

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

                refactor_engine.refactor_imports(
                    &crate_paths,
                    imports,
                    &workspace_root,
                )?;
            },
        }

        if !quiet {
            if dry_run {
                println!("✅ Dry run completed successfully!");
            } else {
                println!("✅ Refactoring completed successfully!");
            }
        }

        Ok((0, crate_paths)) // TODO: Return actual import count
    }
}

/// Builder for RefactorConfig to make it easier to construct
pub struct RefactorConfigBuilder {
    crate_names: Option<CrateNames>,
    workspace_root: Option<std::path::PathBuf>,
    dry_run: bool,
    verbose: bool,
    quiet: bool,
}

impl RefactorConfigBuilder {
    pub fn new() -> Self {
        Self {
            crate_names: None,
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
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
        })
    }
}

impl Default for RefactorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
