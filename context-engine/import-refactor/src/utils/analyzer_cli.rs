use crate::utils::refactoring_analyzer::{RefactoringAnalyzer, AnalysisConfig};
use std::path::PathBuf;

/// Command-line interface for the duplication analyzer
pub fn run_analyzer(workspace_path: Option<PathBuf>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_path.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    // Validate workspace
    if !workspace_root.exists() {
        return Err(format!("Workspace path does not exist: {}", workspace_root.display()).into());
    }

    // Check if it's a Rust project
    let has_cargo_toml = workspace_root.join("Cargo.toml").exists();
    let has_src_dir = workspace_root.join("src").exists();
    
    if !has_cargo_toml && !has_src_dir {
        // Check for Rust workspace (multiple crates)
        let has_rust_crates = std::fs::read_dir(&workspace_root)?
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                let path = entry.path();
                path.is_dir() && path.join("Cargo.toml").exists() && path.join("src").exists()
            });
            
        if !has_rust_crates {
            return Err("Directory does not appear to be a Rust project (no Cargo.toml or src/ found)".into());
        }
    }

    let config = AnalysisConfig {
        workspace_name: None,
        min_duplicate_threshold: 2,
        complexity_threshold: 3, // Lower threshold for better detection
        similarity_threshold: 0.8,
        verbose,
    };

    let analyzer = RefactoringAnalyzer::new(config);
    analyzer.analyze_and_recommend(&workspace_root)?;

    Ok(())
}