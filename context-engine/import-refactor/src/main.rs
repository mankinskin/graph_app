use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod crate_analyzer;
mod import_parser;
mod refactor_engine;

use crate_analyzer::CrateAnalyzer;
use import_parser::ImportParser;
use refactor_engine::RefactorEngine;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the source crate (A) that will export items
    #[arg(short = 'a', long)]
    source_crate: String,

    /// Name of the target crate (B) that imports from source crate
    #[arg(short = 'b', long)]
    target_crate: String,

    /// Workspace root directory
    #[arg(short = 'w', long, default_value = ".")]
    workspace_root: PathBuf,

    /// Dry run - show what would be changed without modifying files
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Import Refactor Tool");
        println!("Source crate (A): {}", args.source_crate);
        println!("Target crate (B): {}", args.target_crate);
        println!("Workspace root: {}", args.workspace_root.display());
        println!("Dry run: {}", args.dry_run);
        println!();
    }

    // Step 1: Analyze the workspace and find the crates
    let analyzer = CrateAnalyzer::new(&args.workspace_root)?;
    let source_crate_path = analyzer.find_crate(&args.source_crate)?;
    let target_crate_path = analyzer.find_crate(&args.target_crate)?;

    if args.verbose {
        println!("Found source crate at: {}", source_crate_path.display());
        println!("Found target crate at: {}", target_crate_path.display());
        println!();
    }

    // Step 2: Parse imports in target crate that reference source crate
    let parser = ImportParser::new(&args.source_crate);
    let imports = parser.find_imports_in_crate(&target_crate_path)?;

    if args.verbose {
        println!(
            "Found {} import statements referencing '{}':",
            imports.len(),
            args.source_crate
        );
        for import in &imports {
            println!(
                "  {} in {}",
                import.import_path,
                import.file_path.display()
            );
        }
        println!();
    }

    if imports.is_empty() {
        println!(
            "No imports found from '{}' in crate '{}'",
            args.source_crate, args.target_crate
        );
        return Ok(());
    }

    // Step 3: Refactor the imports
    let mut engine =
        RefactorEngine::new(&args.source_crate, args.dry_run, args.verbose);
    engine.refactor_imports(&source_crate_path, &target_crate_path, imports)?;

    if args.dry_run {
        println!("\nDry run completed. No files were modified.");
    } else {
        println!("\nRefactoring completed successfully!");
    }

    Ok(())
}
