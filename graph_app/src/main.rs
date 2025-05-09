#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::obfuscated_if_else)]

mod app;
use app::App;
mod examples;
mod graph;

pub use tracing::*;
pub use std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), eframe::Error>
{
    //let graph = seqraph::gen_graph().unwrap_or_else(|g| g);
    //let app = app::App::new(graph);
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|creation_context|
            Ok(Box::new(
                App::new(creation_context)
            ))
        ),
    )
}
