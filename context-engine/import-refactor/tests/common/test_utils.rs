use anyhow::{
    Context,
    Result,
};
use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};
use tempfile::TempDir;

use import_refactor::{
    crate_analyzer::CrateAnalyzer,
    import_parser::ImportParser,
    refactor_engine::RefactorEngine,
};

use super::ast_analysis::{
    analyze_ast,
    AstAnalysis,
};

/// Test configuration for common test scenarios
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: &'static str,
    pub description: &'static str,
    pub source_crate: &'static str,
    pub target_crate: &'static str,
    pub fixture_name: &'static str,
    pub expected_changes: Option<ExpectedChanges>,
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
    pub temp_dir: TempDir,
    pub source_crate_path: PathBuf,
    pub target_crate_path: PathBuf,
    pub workspace_path: PathBuf,
}

/// Results from running the refactor tool
#[derive(Debug)]
pub struct RefactorResult {
    pub success: bool,
    pub source_analysis_before: AstAnalysis,
    pub source_analysis_after: AstAnalysis,
    pub target_analysis_before: Option<AstAnalysis>,
    pub target_analysis_after: Option<AstAnalysis>,
    pub compilation_results: CompilationResults,
}

/// Compilation test results for both crates
#[derive(Debug)]
pub struct CompilationResults {
    pub source_compiles: bool,
    pub target_compiles: bool,
    pub source_errors: Option<String>,
    pub target_errors: Option<String>,
}

impl TestWorkspace {
    /// Create a protected test workspace from fixture data
    pub fn setup(fixture_name: &str) -> Result<Self> {
        let temp_dir = tempfile::tempdir()
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

        // Find crate paths (will be populated by scenario)
        let source_crate_path = workspace_path.clone();
        let target_crate_path = workspace_path.clone();

        Ok(Self {
            temp_dir,
            source_crate_path,
            target_crate_path,
            workspace_path,
        })
    }

    /// Execute refactor tool with comprehensive validation
    pub fn run_refactor_with_validation(
        &mut self,
        scenario: &TestScenario,
    ) -> Result<RefactorResult> {
        // Update crate paths based on scenario
        let analyzer = CrateAnalyzer::new(&self.workspace_path)?;
        self.source_crate_path = analyzer
            .find_crate(scenario.source_crate)
            .context("Failed to find source crate")?;
        self.target_crate_path = analyzer
            .find_crate(scenario.target_crate)
            .context("Failed to find target crate")?;

        // Analyze initial state
        let source_lib_path = self.source_crate_path.join("src").join("lib.rs");
        let target_lib_path = self.target_crate_path.join("src").join("lib.rs");
        let target_main_path =
            self.target_crate_path.join("src").join("main.rs");

        let source_analysis_before = analyze_ast(&source_lib_path)
            .context("Failed to analyze source crate before refactoring")?;

        let target_analysis_before = if target_lib_path.exists() {
            Some(analyze_ast(&target_lib_path)?)
        } else if target_main_path.exists() {
            Some(analyze_ast(&target_main_path)?)
        } else {
            None
        };

        // Run refactor tool
        let refactor_success = self.execute_refactor(scenario).is_ok();

        // Analyze final state
        let source_analysis_after = analyze_ast(&source_lib_path)
            .context("Failed to analyze source crate after refactoring")?;

        let target_analysis_after = if target_lib_path.exists() {
            Some(analyze_ast(&target_lib_path)?)
        } else if target_main_path.exists() {
            Some(analyze_ast(&target_main_path)?)
        } else {
            None
        };

        // Test compilation
        let compilation_results = self.verify_compilation(scenario)?;

        Ok(RefactorResult {
            success: refactor_success,
            source_analysis_before,
            source_analysis_after,
            target_analysis_before,
            target_analysis_after,
            compilation_results,
        })
    }

    /// Execute the refactor tool
    fn execute_refactor(
        &self,
        scenario: &TestScenario,
    ) -> Result<()> {
        let parser = ImportParser::new(scenario.source_crate);
        let imports = parser.find_imports_in_crate(&self.target_crate_path)?;

        let mut engine =
            RefactorEngine::new(scenario.source_crate, false, false);
        engine.refactor_imports(
            &self.source_crate_path,
            &self.target_crate_path,
            imports,
            &self.workspace_path,
        )?;

        Ok(())
    }

    /// Verify both crates compile after refactoring
    fn verify_compilation(
        &self,
        scenario: &TestScenario,
    ) -> Result<CompilationResults> {
        let source_result = self.check_crate_compilation(scenario.source_crate);
        let target_result = self.check_crate_compilation(scenario.target_crate);

        Ok(CompilationResults {
            source_compiles: source_result.as_ref().map_or(false, |r| r.0),
            target_compiles: target_result.as_ref().map_or(false, |r| r.0),
            source_errors: source_result.ok().and_then(|(_, err)| err),
            target_errors: target_result.ok().and_then(|(_, err)| err),
        })
    }

    /// Check if a specific crate compiles
    fn check_crate_compilation(
        &self,
        crate_name: &str,
    ) -> Result<(bool, Option<String>)> {
        let analyzer = CrateAnalyzer::new(&self.workspace_path)?;
        let crate_path = analyzer.find_crate(crate_name)?;

        let output = Command::new("cargo")
            .arg("check")
            .arg("--quiet")
            .current_dir(&crate_path)
            .output()
            .context("Failed to execute cargo check")?;

        let success = output.status.success();
        let errors = if success {
            None
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(format!("STDERR:\n{}\nSTDOUT:\n{}", stderr, stdout))
        };

        Ok((success, errors))
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

/// Legacy helper function for backward compatibility
/// Prefer using TestWorkspace::setup() for new tests
pub fn setup_test_workspace(fixture_name: &str) -> Result<TempDir> {
    let workspace = TestWorkspace::setup(fixture_name)?;
    Ok(workspace.temp_dir)
}

/// Legacy helper function for backward compatibility
/// Prefer using TestWorkspace::run_refactor_with_validation() for new tests
pub fn run_refactor(
    workspace_path: &Path,
    source_crate: &str,
    target_crate: &str,
) -> Result<()> {
    let analyzer = CrateAnalyzer::new(workspace_path)?;
    let source_crate_path = analyzer.find_crate(source_crate)?;
    let target_crate_path = analyzer.find_crate(target_crate)?;

    let parser = ImportParser::new(source_crate);
    let imports = parser.find_imports_in_crate(&target_crate_path)?;

    let mut engine = RefactorEngine::new(source_crate, false, false);
    engine.refactor_imports(
        &source_crate_path,
        &target_crate_path,
        imports,
        workspace_path,
    )?;

    Ok(())
}

/// Legacy helper function for backward compatibility
/// Prefer using TestWorkspace::verify_compilation() for new tests  
pub fn check_compilation(
    workspace_path: &Path,
    crate_name: &str,
) -> Result<bool> {
    let analyzer = CrateAnalyzer::new(workspace_path)?;
    let crate_path = analyzer.find_crate(crate_name)?;

    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&crate_path)
        .output()?;

    if !output.status.success() {
        println!("Compilation failed for {}:", crate_name);
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output.status.success())
}
