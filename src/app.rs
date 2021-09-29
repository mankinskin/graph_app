use eframe::{egui::{self, DragValue, Ui}, epi};
#[cfg(feature = "persistence")]
use serde::*;
use std::{
    sync::{Arc, RwLock},
    num::NonZeroUsize,
};

#[allow(unused)]
use crate::graph::*;

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    label: String,

    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    // this how you opt-out of serialization of a member
    graph: Graph,
    split_lower: usize,
    split_upper: usize,
    insert_text: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            graph_file: None,
            graph: Graph::new(),
            split_lower: 1,
            split_upper: 7,
            insert_text: Default::default(),
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.menu("File", |ui| {
                if ui.button("Open...").clicked() {
                    self.open_file_dialog();
                } else if ui.button("Quit").clicked() {
                    frame.quit();
                }
            });
        });
    }
    fn side_panel(&mut self, ctx: &egui::CtxRef) {
        egui::SidePanel::left("side_panel")
            .show(ctx, |ui| {
                ui.heading("Side Panel");
                ui.horizontal(|ui| {
                    ui.label("Write something: ");
                    ui.text_edit_singleline(&mut self.label);
                });
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }
    fn central_panel(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.graph.show(ui);
                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }
    fn context_menu(&mut self, ui: &mut Ui) {
        //ui.horizontal(|ui| {
        //    ui.label("Insert:");
        //    ui.text_edit_singleline(self.insert_text);
        //    if ui.button("Go").clicked() {
        //        self.graph.read(self.insert_text);
        //        self.insert_text = String::new();
        //        ui.close();
        //    }
        //});
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.split_lower));
            ui.add(DragValue::new(&mut self.split_upper));
            self.split_upper = self.split_lower.max(self.split_upper);
            if ui.button("Split").clicked() {
                match (NonZeroUsize::new(self.split_lower), NonZeroUsize::new(self.split_upper)) {
                    (Some(lower), Some(upper)) => self.graph.split_range(lower, upper),
                    (None, Some(single)) |
                    (Some(single), None) => self.graph.split(single),
                    (None, None) => {},
                }
                ui.close();
            }
        });
        if ui.button("Reset").clicked() {
            self.graph.reset();
            ui.close();
        }
        ui.menu("Layout", |ui| {
            if ui.radio_value(
                self.graph.get_layout_mut(),
                Layout::Graph, "Graph"
                )
                .clicked() {
                ui.close();
            }
            if ui.radio_value(
                self.graph.get_layout_mut(),
                Layout::Nested, "Nested"
                )
                .clicked() {
                ui.close();
            }
        });
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
        //self.side_panel(ctx);
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
