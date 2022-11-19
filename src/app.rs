use crate::*;
use eframe::{
    egui::{
        self,
        Ui,
    },
};
use crate::examples::*;
use seqraph::HypergraphRef;
#[cfg(feature = "persistence")]
use serde::*;
use tokio::task::JoinHandle;

#[allow(unused)]
use crate::graph::*;

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Graph,
    inserter: bool,
    read_task: Option<JoinHandle<()>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            graph_file: None,
            graph: Graph::new(),
            inserter: true,
            read_task: None,
        }
    }
    #[allow(unused)]
    pub fn from_graph_ref(graph: HypergraphRef<char>) -> Self {
        Self {
            graph_file: None,
            graph: Graph::new_from_graph_ref(graph),
            inserter: true,
            read_task: None,
        }
    }
    pub fn context_menu(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Quick Insert:");
            ui.text_edit_singleline(&mut self.graph.insert_text);
            if ui.button("Go").clicked() {
                let insert_text = self.graph.insert_text.clone();
                self.graph.read_text(insert_text, ui.ctx());
                self.graph.insert_text = String::new();
                ui.close_menu();
            }
        });
        if ui.button("Open Inserter").clicked() {
            self.inserter = true;
            ui.close_menu();
        }
        {
            let mut vis = self.graph.vis_mut();
            ui.menu_button("Layout", |ui| {
                ui.radio_value(&mut vis.layout, Layout::Graph, "Graph")
                    .clicked();
                ui.radio_value(&mut vis.layout, Layout::Nested, "Nested")
                    .clicked();
            });
        }
        ui.menu_button("Load preset...", |ui| {
            if ui.button("Graph 1").clicked() {
                self.graph.set_graph(build_graph1());
                ui.close_menu();
            }
            if ui.button("Graph 2").clicked() {
                self.graph.set_graph(build_graph2());
                ui.close_menu();
            }
            if ui.button("Graph 3").clicked() {
                self.graph.set_graph(build_graph3());
                ui.close_menu();
            }
        });
        if ui.button("Clear").clicked() {
            self.graph.clear();
            ui.close_menu();
        }
    }
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
    fn top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Actions...", |ui| {
                    self.context_menu(ui)
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close()
                    }
                })
            })
        });
    }
    fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.graph.show(ui);
                //tracing_egui::show(ui.ctx(), &mut true);
                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| {
                self.context_menu(ui)
            });
    }
}
impl eframe::App for App {
    fn max_size_points(&self) -> egui::Vec2 {
        (f32::INFINITY, f32::INFINITY).into()
    }
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.top_panel(ctx, frame);
        self.central_panel(ctx);
        if self.inserter {
            egui::Window::new("Inserter")
                .show(ctx, |ui| {
                    ui.text_edit_multiline(&mut self.graph.insert_text);
                    if ui.button("Insert").clicked() {
                        let insert_text = self.graph.insert_text.clone();
                        self.read_task = Some(self.graph.read_text(insert_text, ctx));
                        self.graph.insert_text = String::new();
                    }
                });
        }
    }
    fn on_close_event(&mut self) -> bool {
        if let Some(handle) = self.read_task.take() {
            println!("abort");
            handle.abort();
        }
        true
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
