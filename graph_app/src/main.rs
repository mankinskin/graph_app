#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::obfuscated_if_else)]

mod app;
use app::App;
mod examples;
mod graph;
mod read;
mod vis;

pub use std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
pub use tracing::*;
#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[command(flatten)]
    rerun: rerun::clap::RerunArgs,
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser as _;

    use crate::graph::Graph;
    let args = Args::parse();

    let (rec, _serve_guard) = args.rerun.init("context_graph_app")?;
    let graph = Graph::from(rec);
    println!("Main thread {:?}", std::thread::current().id());
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|creation_context| {
            use eframe::egui::ThemePreference;

            creation_context.egui_ctx.set_theme(ThemePreference::Dark);
            Ok(Box::new(App::from(graph)))
        }),
    )
    .map_err(|e| e.into())
}
