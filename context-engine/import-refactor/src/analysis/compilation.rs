use anyhow::{
    Context,
    Result,
};
use std::{
    path::Path,
    process::Command,
};

/// Results from compilation testing
#[derive(Debug)]
pub struct CompilationResults {
    pub source_compiles: bool,
    pub target_compiles: bool,
    pub source_errors: Option<String>,
    pub target_errors: Option<String>,
}

/// Compilation checker for validating refactor results
pub struct CompilationChecker;

impl CompilationChecker {
    pub fn check_crates(
        source_crate_path: &Path,
        target_crate_path: &Path,
    ) -> Result<CompilationResults> {
        let source_result = Self::check_crate_compilation(source_crate_path);
        let target_result = Self::check_crate_compilation(target_crate_path);

        Ok(CompilationResults {
            source_compiles: source_result.as_ref().is_ok_and(|r| r.0),
            target_compiles: target_result.as_ref().is_ok_and(|r| r.0),
            source_errors: source_result.ok().and_then(|(_, err)| err),
            target_errors: target_result.ok().and_then(|(_, err)| err),
        })
    }

    /// Check if a specific crate compiles
    fn check_crate_compilation(
        crate_path: &Path
    ) -> Result<(bool, Option<String>)> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--quiet")
            .current_dir(crate_path)
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

    /// Format compilation errors for display
    pub fn format_compilation_errors(results: &CompilationResults) -> String {
        let mut output = String::new();

        if !results.source_compiles {
            output.push_str("🔴 Source crate compilation failed:\n");
            if let Some(errors) = &results.source_errors {
                output.push_str(errors);
            }
            output.push('\n');
        }

        if !results.target_compiles {
            output.push_str("🔴 Target crate compilation failed:\n");
            if let Some(errors) = &results.target_errors {
                output.push_str(errors);
            }
            output.push('\n');
        }

        if results.source_compiles && results.target_compiles {
            output.push_str("✅ Both crates compile successfully\n");
        }

        output
    }
}
