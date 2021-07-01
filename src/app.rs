use eframe::{egui, epi};
use std::sync::{
    Arc,
    RwLock,
};
use petgraph::graph::DiGraph;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    label: String,

    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Arc<RwLock<Option<String>>>,
    open: bool,
    test: DiGraph<(), ()>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            graph_file: None,
            graph: Arc::new(RwLock::new(None)),
            test: DiGraph::new(),
            open: true,
        }
    }
}
impl TemplateApp {
    async fn read_graph_file(graph: Arc<RwLock<Option<String>>>, file: &rfd::FileHandle) {
        let content = file.read().await;
        match std::str::from_utf8(&content[..]) {
            Ok(content) => { *graph.write().unwrap() = Some(content.to_string()); }
            Err(err) => {
                rfd::AsyncMessageDialog::default()
                    .set_description(&format!("{}", err))
                    .show()
                    .await;
            }
        }
    }
}
impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "egui template"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, storage: Option<&dyn epi::Storage>) {
        if let Some(storage) = storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            self.open = true;
            //if let Some(path) = self.graph_file.clone() {
            //    let graph_ref = self.graph.clone();
            //    execute(async {
            //        Self::read_graph_file(graph_ref, &file).await;
            //    });
            //}
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Open...").clicked() {
                        // open graph file
                        let mut dialog = rfd::AsyncFileDialog::default();
                        let res = std::env::current_dir();
                        if let Ok(current_dir) = &res {
                            dialog = dialog.set_directory(current_dir);
                        }
                        let graph_ref = self.graph.clone();
                        execute(async {
                            if let Some(file) = dialog.pick_file().await {
                                Self::read_graph_file(graph_ref, &file).await;
                                //self.graph_file = Some(file.);
                            }
                        });
                    } else if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            if let Some(graph) = &*self.graph.read().unwrap() {
                egui::Window::new("code")
                    .open(&mut self.open)
                    .scroll(true)
                    .resizable(true)
                    //.default_pos((0.0, 0.0))
                    .default_height(10.0)
                    //.default_width(10.0)
                    .show(ctx, |ui| {
                        ui.label(
                            "This window is resizable and has a scroll area. You can shrink it to any size",
                        );
                        ui.separator();
                        ui.code(graph);
                    });
            } else {
                ui.heading("No file loaded");   
            }
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}