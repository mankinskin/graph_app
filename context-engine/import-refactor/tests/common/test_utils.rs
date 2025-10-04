use anyhow::Result;
use std::{
    fs,
    path::Path,
};
use tempfile::TempDir;

use import_refactor::{
    crate_analyzer::CrateAnalyzer,
    import_parser::ImportParser,
    refactor_engine::RefactorEngine,
};

/// Test configuration for common test scenarios
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: &'static str,
    pub description: &'static str,
    pub source_crate: &'static str,
    pub target_crate: &'static str,
    pub fixture_name: &'static str,
}

/// Common test scenarios that can be reused
pub const TEST_SCENARIOS: &[TestScenario] = &[
    TestScenario {
        name: "basic_refactoring",
        description: "Basic import refactoring with nested modules",
        source_crate: "source_crate",
        target_crate: "target_crate",
        fixture_name: "basic_workspace",
    },
    TestScenario {
        name: "macro_handling",
        description: "Handling macro exports and conditional compilation",
        source_crate: "macro_source",
        target_crate: "macro_target",
        fixture_name: "macro_workspace",
    },
];

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

/// Helper function to create a test workspace from a fixture
pub fn setup_test_workspace(fixture_name: &str) -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(fixture_name);

    copy_dir_all(&fixture_path, temp_dir.path())?;

    // Find all subdirectories that contain a Cargo.toml (these are workspace members)
    let mut members = Vec::new();
    for entry in fs::read_dir(temp_dir.path())? {
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

    // Create a workspace Cargo.toml at the root
    let workspace_toml = temp_dir.path().join("Cargo.toml");
    let workspace_content = format!(
        "[workspace]\nresolver = \"2\"\nmembers = [{}]\n",
        members.join(", ")
    );
    fs::write(&workspace_toml, workspace_content)?;

    Ok(temp_dir)
}

/// Helper function to run the refactor tool on a workspace
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

    let mut engine = RefactorEngine::new(source_crate, false, true);
    engine.refactor_imports(
        &source_crate_path,
        &target_crate_path,
        imports,
        workspace_path,
    )?;

    Ok(())
}

/// Helper function to check compilation
pub fn check_compilation(
    workspace_path: &Path,
    crate_name: &str,
) -> Result<bool> {
    let analyzer = CrateAnalyzer::new(workspace_path)?;
    let crate_path = analyzer.find_crate(crate_name)?;

    let output = std::process::Command::new("cargo")
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
