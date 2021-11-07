use eframe::{egui, epi};
#[cfg(feature = "persistence")]
use serde::*;
use std::sync::{Arc, RwLock};

#[allow(unused)]
use crate::graph::*;

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Graph,
}

impl Default for App {
    fn default() -> Self {
        Self {
            graph_file: None,
            graph: Graph::new(),
        }
    }
}
impl App {
    #[allow(unused)]
    async fn read_graph_file(graph: Arc<RwLock<Option<String>>>, file: &rfd::FileHandle) {
        let content = file.read().await;
        match std::str::from_utf8(&content[..]) {
            Ok(content) => {}
            Err(err) => {
                rfd::AsyncMessageDialog::default()
                    .set_description(&format!("{}", err))
                    .show()
                    .await;
            }
        }
    }
    #[allow(unused)]
    fn open_file_dialog(&mut self) {
        // open graph file
        let mut dialog = rfd::AsyncFileDialog::default();
        let res = std::env::current_dir();
        if let Ok(current_dir) = &res {
            dialog = dialog.set_directory(current_dir);
        }
    }
    fn top_panel(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui|
            ui.horizontal(|ui| {
                ui.menu_button("Actions...", |ui| {
                    self.graph.context_menu(ui)
                });
                ui.with_layout(egui::Layout::right_to_left(), |ui|
                    if ui.button("Quit").clicked() {
                        frame.quit()
                    }
                )
            })
        );
    }
    fn central_panel(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.graph.show(ui);
                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.graph.context_menu(ui));
    }
}
impl epi::App for App {
    fn name(&self) -> &str {
        "Graph App"
    }

    /// Called by the framework to load old app state (if any).
    #[allow(unused)]
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        #[cfg(feature = "persistence")]
        if let Some(storage) = storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        }
    }
    fn max_size_points(&self) -> egui::Vec2 {
        (f32::INFINITY, f32::INFINITY).into()
    }
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.top_panel(ctx, frame);
        self.central_panel(ctx);
    }
}
use std::future::Future;
#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    tokio::spawn(f);
}
#[allow(unused)]
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
