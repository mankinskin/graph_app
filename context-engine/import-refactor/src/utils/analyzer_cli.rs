use crate::utils::{
    refactoring_analyzer::{RefactoringAnalyzer, AnalysisConfig},
    duplication_analyzer::{CodebaseDuplicationAnalyzer, AnalysisConfig as DuplicationConfig, AiProvider},
};
use std::path::PathBuf;

/// Command-line interface for the duplication analyzer
pub fn run_analyzer(workspace_path: Option<PathBuf>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_analyzer_with_ai(workspace_path, verbose, false, AiProvider::Auto, None, 20))
}

/// Command-line interface for the duplication analyzer with AI support
pub async fn run_analyzer_with_ai(
    workspace_path: Option<PathBuf>, 
    verbose: bool,
    enable_ai: bool,
    ai_provider: AiProvider,
    ai_model: Option<String>,
    max_functions_for_ai: usize,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // Run enhanced duplication analysis
    run_duplication_analysis(
        &workspace_root, 
        verbose, 
        enable_ai, 
        &ai_provider,  // Pass by reference
        ai_model, 
        max_functions_for_ai
    ).await?;

    // Also run the existing refactoring analyzer for additional insights
    if verbose {
        println!("\n🔄 Running additional refactoring analysis...");
        run_legacy_analysis(&workspace_root, verbose).await?;
    }

    Ok(())
}

/// Run the new AI-powered duplication analysis
async fn run_duplication_analysis(
    workspace_root: &PathBuf,
    _verbose: bool,
    enable_ai: bool,
    ai_provider: &AiProvider,
    ai_model: Option<String>,
    max_functions_for_ai: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = DuplicationConfig {
        min_complexity_threshold: 3,
        similarity_threshold: 0.8,
        min_function_length: 3,
        exclude_patterns: vec![
            "test".to_string(),
            "tests".to_string(),
            "target".to_string(),
            ".git".to_string(),
        ],
        max_files_to_scan: None,
        enable_ai_analysis: enable_ai,
        ai_api_key: None, // Will be read from environment
        ai_provider: ai_provider.clone(),
        ai_model,
        max_functions_for_ai,
    };

    let mut analyzer = CodebaseDuplicationAnalyzer::with_config(workspace_root, config);
    
    if enable_ai {
        println!("🤖 AI-powered analysis enabled (provider: {:?})", ai_provider);
        println!("⚠️  Note: This will make API calls and may incur costs");
    }
    
    let analysis = analyzer.analyze_codebase().await?;
    analyzer.print_analysis_results(&analysis);

    Ok(())
}

/// Run the legacy refactoring analyzer (for compatibility)
async fn run_legacy_analysis(workspace_root: &PathBuf, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = AnalysisConfig {
        workspace_name: None,
        min_duplicate_threshold: 2,
        complexity_threshold: 3,
        similarity_threshold: 0.8,
        verbose,
    };

    let analyzer = RefactoringAnalyzer::new(config);
    analyzer.analyze_and_recommend(workspace_root).await?;

    Ok(())
}