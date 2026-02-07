#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod algorithm;
mod app;
pub use app::App;
mod examples;
mod graph;
mod output;
mod read;
pub(crate) mod task;
mod vis;
pub(crate) mod widgets;

// ----------------------------------------------------------------------------
// When compiling for web:

#[allow(unused)]
use crate::graph::Graph;
#[cfg(not(target_arch = "wasm32"))]
use context_trace::graph::HypergraphRef;
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{
    self,
    prelude::*,
};
pub(crate) use tracing::*;


// Native-only open function
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn open(_graph: HypergraphRef) -> Result<(), eframe::Error> {
    // Each tab manages its own graph internally
    let app = App::new();
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(app))),
    )
}
