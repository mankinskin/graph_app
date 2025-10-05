//! High-level API for running import refactoring operations.
//!
//! This module provides a unified interface for import refactoring that can be used
//! by both the CLI application and test frameworks, eliminating code duplication.

use anyhow::Result;
use std::path::Path;

use crate::{
    crate_analyzer::{
        CrateAnalyzer,
        CrateNames,
        CratePaths,
    },
    import_parser::ImportParser,
    refactor_engine::RefactorEngine,
    utils::common::format_relative_path,
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
                        "📦 Source crate (A): {} → will export items via pub use",
                        source_crate
                    );
                    println!(
                        "📦 Target crate (B): {} → imports will be simplified to use A::*",
                        target_crate
                    );
                },
            }
            if dry_run {
                println!(
                    "🔍 Running in dry-run mode (no files will be modified)"
                );
            }
            println!("📂 Workspace: {}", workspace_root.display());
            println!();
        }

        // Step 1: Analyze the workspace and find the crates
        let analyzer = CrateAnalyzer::new(&workspace_root)?;
        let paths = analyzer.find_crates(&crate_names)?;

        if verbose && !quiet {
            paths.print_found(&workspace_root);
            println!();
        }

        // Step 2: Parse imports
        let imports = Self::collect_imports(&crate_names, &paths)?;

        if !quiet {
            Self::print_import_scan_results(&crate_names, &imports);
        }

        if imports.is_empty() {
            if !quiet {
                Self::print_no_imports_message(&crate_names);
            }
            return Ok((0, paths));
        }

        if verbose && !quiet {
            println!("\n📝 Detailed import list:");
            for import in &imports {
                println!(
                    "  • {} in {}",
                    import.import_path,
                    format_relative_path(&import.file_path, &workspace_root)
                );
            }
            println!();
        }

        // Step 3: Execute the refactoring
        let imports_count = imports.len();
        let mut engine = RefactorEngine::new(&crate_names, dry_run, verbose);
        engine.refactor_imports(&paths, imports, &workspace_root)?;

        if !quiet {
            Self::print_completion_message(&crate_names, dry_run);
        }

        Ok((imports_count, paths))
    }

    /// Collect imports based on the refactoring mode
    fn collect_imports(
        crate_names: &CrateNames,
        paths: &CratePaths,
    ) -> Result<Vec<crate::import_parser::ImportInfo>> {
        match crate_names {
            CrateNames::SelfRefactor { crate_name } => {
                // For self-refactor, we need to collect both crate:: imports and external imports
                let crate_path = match paths {
                    CratePaths::SelfRefactor { crate_path } => crate_path,
                    _ => unreachable!("Mismatched crate names and paths"),
                };

                // Collect crate:: imports
                let crate_parser = ImportParser::new("crate");
                let crate_imports =
                    crate_parser.find_imports_in_crate(crate_path)?;

                // Collect external imports that reference the same crate
                let external_parser = ImportParser::new(crate_name);
                let mut external_imports =
                    external_parser.find_imports_in_crate(crate_path)?;

                // Normalize external imports to crate:: format to avoid duplicates
                for import in &mut external_imports {
                    let crate_name_prefix = format!("{}::", crate_name);
                    if import.import_path.starts_with(&crate_name_prefix) {
                        import.import_path = import
                            .import_path
                            .replace(&crate_name_prefix, "crate::");
                    }

                    // Also normalize the imported items
                    for item in &mut import.imported_items {
                        if item.starts_with(&crate_name_prefix) {
                            *item = item.replace(&crate_name_prefix, "crate::");
                        }
                    }
                }

                // Combine both types of imports
                let mut imports = crate_imports;
                imports.extend(external_imports);
                Ok(imports)
            },
            CrateNames::CrossRefactor { source_crate, .. } => {
                // For cross-refactor, use the new unified method
                let parser = ImportParser::new(source_crate);
                parser.find_imports_in_crates(paths)
            },
        }
    }

    /// Print the import scanning results
    fn print_import_scan_results(
        crate_names: &CrateNames,
        imports: &[crate::import_parser::ImportInfo],
    ) {
        let (source_desc, target) = match crate_names {
            CrateNames::SelfRefactor { crate_name } => (
                format!("'crate::' and '{}::'", crate_name),
                crate_name.as_str(),
            ),
            CrateNames::CrossRefactor {
                source_crate,
                target_crate,
            } => (format!("'{}'", source_crate), target_crate.as_str()),
        };

        println!("🔎 Scanning for {} imports in '{}'...", source_desc, target);
        println!("✅ Found {} import statements", imports.len());
    }

    /// Print message when no imports are found
    fn print_no_imports_message(crate_names: &CrateNames) {
        let (source_desc, target) = match crate_names {
            CrateNames::SelfRefactor { crate_name } => (
                format!("'crate::' or '{}::'", crate_name),
                crate_name.as_str(),
            ),
            CrateNames::CrossRefactor {
                source_crate,
                target_crate,
            } => (format!("'{}'", source_crate), target_crate.as_str()),
        };

        println!("❌ No {} imports found in crate '{}'", source_desc, target);
        println!("   Nothing to refactor.");
    }

    /// Print completion message
    fn print_completion_message(
        crate_names: &CrateNames,
        dry_run: bool,
    ) {
        if dry_run {
            println!("🔍 Dry run completed. No files were modified.");
            println!("💡 Run without --dry-run to apply these changes.");
        } else {
            let modified = match crate_names {
                CrateNames::SelfRefactor { crate_name } =>
                    format!("'{}'", crate_name),
                CrateNames::CrossRefactor {
                    source_crate,
                    target_crate,
                } => format!("both '{}' and '{}'", source_crate, target_crate),
            };
            println!("✅ Refactoring completed successfully!");
            println!("📁 Modified files in {}", modified);
        }
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
