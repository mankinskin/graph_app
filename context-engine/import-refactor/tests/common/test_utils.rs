use anyhow::{
    Context,
    Result,
};
use import_refactor::{
    crate_analyzer::{
        CrateAnalyzer,
        CrateNames,
        CratePaths,
    },
    refactor_api::{
        RefactorApi,
        RefactorConfigBuilder,
    },
};
use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};
use tempfile::TempDir;

use super::ast_analysis::{
    analyze_ast,
    AstAnalysis,
};

/// Test configuration for common test scenarios
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: &'static str,
    pub description: &'static str,
    pub crate_names: CrateNames,
    pub fixture_name: &'static str,
    pub expected_changes: Option<ExpectedChanges>,
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
}

/// Expected changes after refactoring for validation
#[derive(Debug, Clone)]
pub struct ExpectedChanges {
    pub source_crate_exports: &'static [&'static str],
    pub target_crate_wildcards: u32,
    pub preserved_macros: &'static [&'static str],
    pub nested_modules: &'static [&'static str],
}

/// Comprehensive test execution context
#[derive(Debug)]
pub struct TestWorkspace {
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub crate_paths: CratePaths,
    pub workspace_path: PathBuf,
}

/// Results from running the refactor tool
#[derive(Debug)]
pub struct RefactorResult {
    pub success: bool,
    pub source_analysis_before: AstAnalysis,
    pub source_analysis_after: AstAnalysis,
    //pub target_analysis_before: Option<AstAnalysis>,
    //pub target_analysis_after: Option<AstAnalysis>,
}

impl TestWorkspace {
    /// Create a protected test workspace from fixture data
    pub fn setup(fixture_name: &str) -> Result<Self> {
        let temp_dir = tempfile::tempdir_in(".")
            .context("Failed to create temporary directory")?;

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

        copy_dir_all(&fixture_path, temp_dir.path())
            .context("Failed to copy fixture to temp workspace")?;

        let workspace_path = temp_dir.path().to_path_buf();

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

        //let target_analysis_after = match &self.crate_paths {
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

        Ok(RefactorResult {
            success: refactor_success,
            source_analysis_before,
            source_analysis_after,
            //target_analysis_before,
            //target_analysis_after,
        })
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
