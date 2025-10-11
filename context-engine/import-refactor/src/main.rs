#![feature(iter_intersperse)]

#[cfg(not(test))]
use anyhow::Result;
#[cfg(not(test))]
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

#[cfg(not(test))]
use cli::args::{
    Args,
    Commands,
};

#[cfg(not(test))]
use cli::commands::{
    download_model,
    init_config,
    list_models,
    run_analysis,
    run_refactor,
    run_server,
};

#[cfg(not(test))]
fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::Imports { .. } => {
            // Import refactor mode
            if let Some(import_args) = args.get_import_args() {
                run_refactor(&import_args)
            } else {
                Err(anyhow::anyhow!("Failed to parse import arguments"))
            }
        },
        Commands::Analyze { .. } => {
            // Analyzer mode: analyze codebase for duplications (async for AI support)
            if let Some(analysis_args) = args.get_analysis_args() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(run_analysis(&analysis_args))
            } else {
                Err(anyhow::anyhow!("Failed to parse analysis arguments"))
            }
        },
        Commands::Serve { .. } => {
            // Server mode: start the Candle LLM server
            if let Some(server_args) = args.get_server_args() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(run_server(&server_args))
            } else {
                Err(anyhow::anyhow!("Failed to parse server arguments"))
            }
        },
        Commands::DownloadModel { model_id } => {
            // Download model mode
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(download_model(model_id))
        },
        Commands::ListModels => {
            // List models mode
            list_models()
        },
        Commands::InitConfig { config_file } => {
            // Initialize configuration
            init_config(config_file.clone())
        },
    }
}
