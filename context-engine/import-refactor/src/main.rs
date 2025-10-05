#![feature(iter_intersperse)]

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod crate_analyzer;
mod import_parser;
mod item_info;
mod refactor_api;
mod refactor_engine;
mod utils;

use refactor_api::{
    RefactorApi,
    RefactorConfigBuilder,
};

use crate::crate_analyzer::CrateNames;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the source crate (A) that will export items
    #[arg(short = 'a', long = "source-crate", alias = "source")]
    source_crate: Option<String>,

    /// Name of the target crate (B) that imports from source crate
    #[arg(short = 'b', long = "target-crate", alias = "target")]
    target_crate: Option<String>,

    /// Self-refactor mode: refactor internal crate:: imports within a single crate
    #[arg(
        long = "self",
        help = "Refactor crate:: imports within the specified crate to root-level exports"
    )]
    self_refactor: bool,

    /// Run duplication analyzer on the codebase
    #[arg(
        long = "analyze",
        help = "Analyze the codebase for duplicate and similar functions"
    )]
    analyze: bool,

    /// Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self
    #[arg(
        help = "Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self"
    )]
    positional: Vec<String>,

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

impl Args {
    /// Get the source crate name, preferring the flag over the positional argument
    fn get_source_crate(&self) -> Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Source crate must be specified either via --source-crate/--source flag or as the first positional argument"))
        }
    }

    /// Get the target crate name, preferring the flag over the positional argument
    fn get_target_crate(&self) -> Result<String> {
        if let Some(target) = &self.target_crate {
            Ok(target.clone())
        } else if self.positional.len() >= 2 {
            Ok(self.positional[1].clone())
        } else {
            Err(anyhow::anyhow!("Target crate must be specified either via --target-crate/--target flag or as the second positional argument"))
        }
    }

    /// Get the crate name for self-refactor mode
    fn get_self_crate(&self) -> Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Crate must be specified either via --source-crate/--source flag or as the first positional argument when using --self"))
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.analyze {
        // Analyzer mode: analyze codebase for duplications
        run_analyzer(&args)
    } else {
        // Import refactor mode (handles both self-refactor and standard modes)
        run_import_refactor(&args)
    }
}

fn run_import_refactor(args: &Args) -> Result<()> {
    let workspace_root = args
        .workspace_root
        .canonicalize()
        .unwrap_or_else(|_| args.workspace_root.clone());

    let crate_names = if args.self_refactor {
        let crate_name = args.get_self_crate()?;
        CrateNames::SelfRefactor { crate_name }
    } else {
        let source_crate = args.get_source_crate()?;
        let target_crate = args.get_target_crate()?;
        CrateNames::CrossRefactor {
            source_crate,
            target_crate,
        }
    };

    let config = RefactorConfigBuilder::new()
        .crate_names(crate_names)
        .workspace_root(workspace_root)
        .dry_run(args.dry_run)
        .verbose(args.verbose)
        .quiet(false)
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

fn run_analyzer(args: &Args) -> Result<()> {
    utils::analyzer_cli::run_analyzer(
        Some(args.workspace_root.clone()),
        args.verbose,
    )
    .map_err(|e| anyhow::anyhow!("Analyzer failed: {}", e))
}
