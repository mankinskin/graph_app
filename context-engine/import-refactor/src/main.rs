#![feature(iter_intersperse)]

use anyhow::Result;
use clap::Parser;

// Import the new modular structure
mod ai;
mod analysis;
mod cli;
mod common;
mod core;
mod io;
mod server;
mod syntax;

use cli::{
    args::Args,
    commands::{
        download_model,
        init_config,
        list_models,
        run_analysis,
        run_refactor,
        run_server,
    },
};

fn main() -> Result<()> {
    let args = Args::parse();

    if args.serve {
        // Server mode: start the Candle LLM server
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_server(&args))
    } else if let Some(model_id) = &args.download_model {
        // Download model mode
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(download_model(&args, model_id))
    } else if args.list_models {
        // List models mode
        list_models()
    } else if args.init_config {
        // Initialize configuration
        init_config(&args)
    } else if args.analyze {
        // Analyzer mode: analyze codebase for duplications (async for AI support)
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_analysis(&args))
    } else {
        // Import refactor mode (handles both self-refactor and standard modes)
        run_refactor(&args)
    }
}
