use anyhow::{Context, Result};
use refactor_tool::{
    analyze_imports, CrateAnalyzer, CrateNames, CratePaths, ImportParser,
    RefactorApi, RefactorConfigBuilder,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use syn::ItemUse;
use tempfile::TempDir;

use super::ast_analysis::{analyze_ast, AstAnalysis};
use super::validation::{AstValidator, TestFormatter};

/// Type alias for custom validation function
type CustomValidationFn = Box<
    dyn FnOnce(
        &TestScenario,
        &TestWorkspace,
        &RefactorResult,
        &super::validation::ValidationResult,
    ) -> Result<()>,
>;

/// Validation strategy for test execution
enum ValidationStrategy {
    /// Standard validation: assert success and validation passed
    Standard,
    /// Custom validation logic provided by the test
    Custom(CustomValidationFn),
}

/// Test configuration for common test scenarios
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: &'static str,
    pub description: &'static str,
    pub crate_names: CrateNames,
    pub fixture_name: &'static str,
    pub expected_changes: Option<ExpectedChanges>,
    pub keep_exports: bool,
}

impl TestScenario {
    /// Create a self-refactor test scenario
    pub fn self_refactor(
        name: &'static str,
        description: &'static str,
        crate_name: &'static str,
        fixture_name: &'static str,
    ) -> Self {
        Self {
            name,
            description,
            crate_names: CrateNames::SelfRefactor {
                crate_name: crate_name.to_string(),
            },
            fixture_name,
            expected_changes: None,
            keep_exports: false,
        }
    }

    /// Create a cross-refactor test scenario
    pub fn cross_refactor(
        name: &'static str,
        description: &'static str,
        source_crate: &'static str,
        target_crate: &'static str,
        fixture_name: &'static str,
    ) -> Self {
        Self {
            name,
            description,
            crate_names: CrateNames::CrossRefactor {
                source_crate: source_crate.to_string(),
                target_crate: target_crate.to_string(),
            },
            fixture_name,
            expected_changes: None,
            keep_exports: false,
        }
    }

    /// Add expected changes to the scenario
    pub fn with_expected_changes(
        mut self,
        expected_changes: ExpectedChanges,
    ) -> Self {
        self.expected_changes = Some(expected_changes);
        self
    }

    /// Configure the scenario to disable export generation
    pub fn keep_exports(
        mut self,
        keep_exports: bool,
    ) -> Self {
        self.keep_exports = keep_exports;
        self
    }

    /// Execute the test scenario with standard validation (success + validation passed)
    pub fn execute(self) -> Result<()> {
        self.execute_with_strategy(ValidationStrategy::Standard)
    }

    /// Execute the test scenario with custom validation logic
    pub fn execute_with_custom_validation<F>(
        self,
        custom_validation: F,
    ) -> Result<()>
    where
        F: FnOnce(
                &TestScenario,
                &TestWorkspace,
                &RefactorResult,
                &super::validation::ValidationResult,
            ) -> Result<()>
            + 'static,
    {
        self.execute_with_strategy(ValidationStrategy::Custom(Box::new(
            custom_validation,
        )))
    }

    /// Execute the test scenario with configurable validation strategy
    fn execute_with_strategy(
        self,
        validation_strategy: ValidationStrategy,
    ) -> Result<()> {
        println!("🚀 Starting test: {}", self.description);

        // Setup protected workspace
        let mut workspace = TestWorkspace::setup(self.fixture_name)?;

        // Run refactor with full validation
        let result = workspace.run_refactor_with_validation(&self)?;

        // Validate results against expectations
        let validation = AstValidator::validate_refactor_result(
            &result,
            self.expected_changes.as_ref(),
        );

        // Format and display comprehensive results
        let formatted_output =
            TestFormatter::format_test_results(self.name, &result, &validation);
        println!("{}", formatted_output);

        // Execute validation strategy
        match validation_strategy {
            ValidationStrategy::Standard => {
                // Show detailed validation failures if any
                if !validation.passed {
                    println!("\n❌ VALIDATION FAILURES DETECTED:");
                    for failure in &validation.failures {
                        println!("   {}", failure);
                    }
                    println!();
                }
                
                // Assert overall success with detailed failure information
                assert!(
                    validation.passed, 
                    "Test validation failed with {} error(s): {}",
                    validation.failures.len(),
                    validation.failures.join("; ")
                );
                assert!(
                    result.success, 
                    "Refactor execution failed - check refactor tool output above for details"
                );
            },
            ValidationStrategy::Custom(custom_validation) => {
                // Show detailed validation failures for custom validation too
                if !validation.passed {
                    println!("\n❌ VALIDATION FAILURES DETECTED:");
                    for failure in &validation.failures {
                        println!("   {}", failure);
                    }
                    println!();
                }
                
                // Execute custom validation logic
                custom_validation(&self, &workspace, &result, &validation)?;
            },
        }

        Ok(())
    }
}

/// Expected changes after refactoring for validation
#[derive(Clone)]
pub struct ExpectedChanges {
    /// Expected pub use structure using syn::ItemUse directly
    pub expected_pub_use: Option<ItemUse>,
    /// Number of wildcard imports expected in target crate
    pub target_crate_wildcards: u32,
    /// Macros that should be preserved (not converted to pub use)
    pub preserved_macros: &'static [&'static str],
}

impl std::fmt::Debug for ExpectedChanges {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ExpectedChanges")
            .field(
                "expected_pub_use",
                &self.expected_pub_use.as_ref().map(|_| "<ItemUse>"),
            )
            .field("target_crate_wildcards", &self.target_crate_wildcards)
            .field("preserved_macros", &self.preserved_macros)
            .finish()
    }
}

impl ExpectedChanges {
    /// Create ExpectedChanges with syn::ItemUse directly
    pub fn with_pub_use(
        expected_pub_use: ItemUse,
        target_wildcards: u32,
        preserved_macros: &'static [&'static str],
    ) -> Self {
        Self {
            expected_pub_use: Some(expected_pub_use),
            target_crate_wildcards: target_wildcards,
            preserved_macros,
        }
    }

    /// Create basic ExpectedChanges without specific pub use expectations
    pub fn basic(
        target_wildcards: u32,
        preserved_macros: &'static [&'static str],
    ) -> Self {
        Self {
            expected_pub_use: None,
            target_crate_wildcards: target_wildcards,
            preserved_macros,
        }
    }
}

/// Test workspace management for isolated testing
#[derive(Debug)]
pub struct TestWorkspace {
    #[allow(dead_code)]
    pub temp_dir: Option<TempDir>,
    pub crate_paths: CratePaths,
    pub workspace_path: PathBuf,
    pub persistent: bool,
}

/// Configuration for test workspace creation
#[derive(Debug, Clone)]
pub struct TestWorkspaceConfig {
    /// Whether to use a persistent directory instead of temp directory
    pub persistent: bool,
    /// Base directory for persistent workspaces (defaults to "./test_workspaces")
    pub persistent_base_dir: Option<PathBuf>,
}

/// Results from running the refactor tool
#[derive(Debug)]
pub struct RefactorResult {
    pub success: bool,
    pub source_analysis_before: AstAnalysis,
    pub source_analysis_after: AstAnalysis,
    pub target_analysis_before: Option<AstAnalysis>,
    pub target_analysis_after: Option<AstAnalysis>,
    pub target_glob_imports_after: u32,
}

impl Default for TestWorkspaceConfig {
    fn default() -> Self {
        Self {
            persistent: false,
            persistent_base_dir: None,
        }
    }
}

impl TestWorkspace {
    /// Create a protected test workspace from fixture data with default config
    pub fn setup(fixture_name: &str) -> Result<Self> {
        Self::setup_with_config(fixture_name, TestWorkspaceConfig::default())
    }

    /// Create a persistent test workspace from fixture data
    pub fn setup_persistent(fixture_name: &str) -> Result<Self> {
        Self::setup_with_config(
            fixture_name,
            TestWorkspaceConfig {
                persistent: true,
                persistent_base_dir: None,
            },
        )
    }

    /// Create a test workspace from fixture data with custom config
    pub fn setup_with_config(
        fixture_name: &str,
        config: TestWorkspaceConfig,
    ) -> Result<Self> {
        let (temp_dir, workspace_path) = if config.persistent {
            // Create persistent directory
            let base_dir = config
                .persistent_base_dir
                .unwrap_or_else(|| PathBuf::from("./test_workspaces"));

            fs::create_dir_all(&base_dir)
                .context("Failed to create persistent base directory")?;

            // Use process ID and timestamp for uniqueness
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let process_id = std::process::id();

            let workspace_path = base_dir
                .join(format!("{}_{}_{}", fixture_name, timestamp, process_id));
            fs::create_dir_all(&workspace_path)
                .context("Failed to create persistent workspace directory")?;

            println!(
                "📁 Created persistent test workspace: {}",
                workspace_path.display()
            );

            (None, workspace_path)
        } else {
            // Create temporary directory
            let temp_dir = tempfile::tempdir_in(".")
                .context("Failed to create temporary directory")?;
            let workspace_path = temp_dir.path().to_path_buf();
            (Some(temp_dir), workspace_path)
        };

        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(fixture_name);

        if !fixture_path.exists() {
            anyhow::bail!(
                "Fixture '{}' not found at {:?}",
                fixture_name,
                fixture_path
            );
        }

        copy_dir_all(&fixture_path, &workspace_path)
            .context("Failed to copy fixture to workspace")?;

        // Create workspace Cargo.toml
        Self::create_workspace_manifest(&workspace_path)?;

        // Initialize with placeholder paths (will be populated by scenario)
        let crate_paths = CratePaths::SelfRefactor {
            crate_path: workspace_path.clone(),
        };

        Ok(Self {
            temp_dir,
            crate_paths,
            workspace_path,
            persistent: config.persistent,
        })
    }

    /// Execute refactor tool with comprehensive validation
    pub fn run_refactor_with_validation(
        &mut self,
        scenario: &TestScenario,
    ) -> Result<RefactorResult> {
        // Debug: Print workspace paths
        println!("🔍 Debug: workspace_path = {:?}", self.workspace_path);

        // Update crate paths based on scenario
        let analyzer = CrateAnalyzer::new(&self.workspace_path)?;
        self.crate_paths = analyzer.find_crates(&scenario.crate_names)?;

        // Debug: Print resolved crate paths
        println!("🔍 Debug: crate_paths = {:?}", self.crate_paths);

        // Analyze initial state
        let source_crate_path = match &self.crate_paths {
            CratePaths::SelfRefactor { crate_path } => crate_path,
            CratePaths::CrossRefactor {
                source_crate_path, ..
            } => source_crate_path,
        };
        let source_lib_path = source_crate_path.join("src").join("lib.rs");

        let source_analysis_before = analyze_ast(&source_lib_path)
            .context("Failed to analyze source crate before refactoring")?;

        //let target_analysis_before = match &self.crate_paths {
        //    CratePaths::SelfRefactor { .. } => None,
        //    CratePaths::CrossRefactor {
        //        target_crate_path, ..
        //    } => {
        //        let target_lib_path =
        //            target_crate_path.join("src").join("lib.rs");
        //        let target_main_path =
        //            target_crate_path.join("src").join("main.rs");

        //        if target_lib_path.exists() {
        //            Some(analyze_ast(&target_lib_path)?)
        //        } else if target_main_path.exists() {
        //            Some(analyze_ast(&target_main_path)?)
        //        } else {
        //            None
        //        }
        //    },
        //};

        // Run refactor tool
        let refactor_result = self.execute_refactor(scenario);
        let refactor_success = refactor_result.is_ok();
        if let Err(e) = &refactor_result {
            eprintln!("🚨 Refactor execution failed: {}", e);
        }

        // Analyze final state
        let source_analysis_after = analyze_ast(&source_lib_path)
            .context("Failed to analyze source crate after refactoring")?;

        // Count glob imports in target crate after refactoring
        let target_glob_imports_after =
            self.count_target_glob_imports(&scenario.crate_names)?;

        Ok(RefactorResult {
            success: refactor_success,
            source_analysis_before,
            source_analysis_after,
            target_analysis_before: None,
            target_analysis_after: None,
            target_glob_imports_after,
        })
    }

    /// Count glob imports in target crate after refactoring
    fn count_target_glob_imports(
        &self,
        crate_names: &CrateNames,
    ) -> Result<u32> {
        let target_crate_path = match &self.crate_paths {
            CratePaths::SelfRefactor { crate_path } => crate_path,
            CratePaths::CrossRefactor {
                target_crate_path, ..
            } => target_crate_path,
        };

        // Use the same logic as the refactor tool to find imports
        let parser = match crate_names {
            CrateNames::SelfRefactor { .. } => ImportParser::new("crate"),
            CrateNames::CrossRefactor { source_crate, .. } => {
                ImportParser::new(source_crate)
            },
        };

        let imports = parser.find_imports_in_crate(target_crate_path)?;
        let analysis_result =
            analyze_imports(&imports, crate_names, &self.workspace_path);

        Ok(analysis_result.glob_imports as u32)
    }

    /// Execute the refactor tool
    fn execute_refactor(
        &self,
        scenario: &TestScenario,
    ) -> Result<()> {
        let config = RefactorConfigBuilder::new()
            .crate_names(scenario.crate_names.clone())
            .workspace_root(&self.workspace_path)
            .dry_run(false)
            .verbose(true)
            .quiet(false) // Keep output for debugging in tests
            .keep_exports(scenario.keep_exports) 
            .build()?;
        let result = RefactorApi::execute_refactor(config);

        if !result.success {
            if let Some(error) = result.error {
                return Err(error);
            } else {
                return Err(anyhow::anyhow!(
                    "Refactoring failed for unknown reason"
                ));
            }
        }

        Ok(())
    }

    /// Create workspace manifest with detected members
    fn create_workspace_manifest(workspace_path: &Path) -> Result<()> {
        let mut members = Vec::new();
        for entry in fs::read_dir(workspace_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let member_path = entry.path();
                let cargo_toml_path = member_path.join("Cargo.toml");
                if cargo_toml_path.exists() {
                    if let Some(dir_name) =
                        member_path.file_name().and_then(|n| n.to_str())
                    {
                        members.push(format!("\"{}\"", dir_name));
                    }
                }
            }
        }

        let workspace_toml = workspace_path.join("Cargo.toml");
        let workspace_content = format!(
            "[workspace]\nresolver = \"2\"\nmembers = [{}]\n",
            members.join(", ")
        );
        fs::write(&workspace_toml, workspace_content)?;

        Ok(())
    }

    /// Get the workspace path for external inspection
    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }

    /// Check if this workspace is persistent (won't be automatically cleaned up)
    pub fn is_persistent(&self) -> bool {
        self.persistent
    }

    /// Manually clean up a persistent workspace (no-op for temp workspaces)
    pub fn cleanup_persistent(&self) -> Result<()> {
        if self.persistent && self.workspace_path.exists() {
            fs::remove_dir_all(&self.workspace_path).with_context(|| {
                format!(
                    "Failed to clean up persistent workspace: {}",
                    self.workspace_path.display()
                )
            })?;
            println!(
                "🗑️  Cleaned up persistent test workspace: {}",
                self.workspace_path.display()
            );
        }
        Ok(())
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        if self.persistent && self.workspace_path.exists() {
            println!(
                "💾 Persistent test workspace preserved at: {}",
                self.workspace_path.display()
            );
            println!(
                "   Use TestWorkspace::cleanup_persistent() to remove manually"
            );
        }
    }
}

/// Helper function to copy a directory recursively
pub fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

///// Legacy helper function for backward compatibility
///// Prefer using TestWorkspace::setup() for new tests
//pub fn setup_test_workspace(fixture_name: &str) -> Result<TempDir> {
//    let workspace = TestWorkspace::setup(fixture_name)?;
//    Ok(workspace.temp_dir)
//}

///// Legacy helper function for backward compatibility
///// Prefer using TestWorkspace::run_refactor_with_validation() for new tests
//pub fn run_refactor(
//    workspace_path: &Path,
//    source_crate: &str,
//    target_crate: &str,
//) -> Result<()> {
//    let crate_names = CrateNames::CrossRefactor {
//        source_crate: source_crate.to_string(),
//        target_crate: target_crate.to_string(),
//    };
//
//    let config = RefactorConfigBuilder::new()
//        .crate_names(crate_names)
//        .workspace_root(workspace_path)
//        .dry_run(false)
//        .verbose(false)
//        .quiet(true)
//        .build()?;
//
//    let result = RefactorApi::execute_refactor(config);
//
//    if !result.success {
//        if let Some(error) = result.error {
//            return Err(error);
//        } else {
//            return Err(anyhow::anyhow!(
//                "Refactoring failed for unknown reason"
//            ));
//        }
//    }
//
//    Ok(())
//}
