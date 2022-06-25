#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod graph;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    //let graph = seqraph::gen_graph().unwrap_or_else(|g| g);
    //let app = app::App::new(graph);
    let app = app::App::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
