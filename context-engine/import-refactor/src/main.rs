use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod crate_analyzer;
mod import_parser;
mod item_info;
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

    println!("🔧 Import Refactor Tool");
    println!(
        "📦 Source crate (A): {} → will export items via pub use",
        args.source_crate
    );
    println!(
        "📦 Target crate (B): {} → imports will be simplified to use A::*",
        args.target_crate
    );
    if args.dry_run {
        println!("🔍 Running in dry-run mode (no files will be modified)");
    }
    println!(
        "📂 Workspace: {}",
        args.workspace_root
            .canonicalize()
            .unwrap_or_else(|_| args.workspace_root.clone())
            .display()
    );
    println!();

    // Step 1: Analyze the workspace and find the crates
    let analyzer = CrateAnalyzer::new(&args.workspace_root)?;
    let source_crate_path = analyzer.find_crate(&args.source_crate)?;
    let target_crate_path = analyzer.find_crate(&args.target_crate)?;

    if args.verbose {
        let workspace_root = args
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| args.workspace_root.clone());
        println!(
            "Found source crate at: {}",
            source_crate_path
                .strip_prefix(&workspace_root)
                .unwrap_or(&source_crate_path)
                .display()
        );
        println!(
            "Found target crate at: {}",
            target_crate_path
                .strip_prefix(&workspace_root)
                .unwrap_or(&target_crate_path)
                .display()
        );
        println!();
    }

    // Step 2: Parse imports in target crate that reference source crate
    let parser = ImportParser::new(&args.source_crate);
    let imports = parser.find_imports_in_crate(&target_crate_path)?;

    println!(
        "🔎 Scanning for imports of '{}' in '{}'...",
        args.source_crate, args.target_crate
    );

    if imports.is_empty() {
        println!(
            "❌ No imports found from '{}' in crate '{}'",
            args.source_crate, args.target_crate
        );
        println!("   Nothing to refactor.");
        return Ok(());
    }

    println!("✅ Found {} import statements", imports.len());

    if args.verbose {
        let workspace_root = args
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| args.workspace_root.clone());
        println!("\n📝 Detailed import list:");
        for import in &imports {
            let relative_path = import
                .file_path
                .strip_prefix(&workspace_root)
                .unwrap_or(&import.file_path);
            println!(
                "  • {} in {}",
                import.import_path,
                relative_path.display()
            );
        }
        println!();
    }

    // Step 3: Refactor the imports
    let mut engine =
        RefactorEngine::new(&args.source_crate, args.dry_run, args.verbose);
    engine.refactor_imports(
        &source_crate_path,
        &target_crate_path,
        imports,
        &args.workspace_root,
    )?;

    if args.dry_run {
        println!("🔍 Dry run completed. No files were modified.");
        println!("💡 Run without --dry-run to apply these changes.");
    } else {
        println!("✅ Refactoring completed successfully!");
        println!(
            "📁 Modified files in both '{}' and '{}'",
            args.source_crate, args.target_crate
        );
    }

    Ok(())
}
