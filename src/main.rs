#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
use app::App;
mod graph;
mod examples;

pub use {
    tracing::*,
};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    //let graph = seqraph::gen_graph().unwrap_or_else(|g| g);
    //let app = app::App::new(graph);
    let app = App::new();
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(app)),
    );
}
