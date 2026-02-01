#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::obfuscated_if_else)]
mod algorithm;
mod app;
pub use algorithm::Algorithm;
pub use app::App;
mod examples;
mod graph;
mod output;
mod read;
mod vis;
pub(crate) mod widgets;

pub use output::OutputBuffer;
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
pub use tracing::*;

pub use std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

// Native-only open function
#[cfg(not(target_arch = "wasm32"))]
pub async fn open(_graph: HypergraphRef) -> Result<(), eframe::Error> {
    // Each tab manages its own graph internally
    let app = App::new();
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(app))),
    )
}
