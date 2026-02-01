#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::obfuscated_if_else)]
mod algorithm;
mod app;
pub use algorithm::Algorithm;
pub use app::App;
mod examples;
mod graph;
mod read;
mod vis;
// ----------------------------------------------------------------------------
// When compiling for web:

use crate::graph::Graph;
use context_trace::graph::HypergraphRef;
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{
    self,
    prelude::*,
};
pub use tracing::*;

pub use {
    std::sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
    //tracing_mutex::{
    //    stdsync::{
    //        TracingRwLock as RwLock,
    //        TracingReadGuard as RwLockReadGuard,
    //        TracingWriteGuard as RwLockWriteGuard,
    //    },
    //},
};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = App::default();
    eframe::start_web(canvas_id, Box::new(app))
}

pub async fn open(_graph: HypergraphRef) -> Result<(), eframe::Error> {
    // Each tab manages its own graph internally
    let app = App::new();
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(app))),
    )
}
