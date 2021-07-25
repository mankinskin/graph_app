#[allow(unused)]
use eframe::{
    egui::{
        self, Color32, Id, Rect, Ui,
        Frame, Style, ComboBox, Area,
        Response, LayerId, CtxRef, Sense,
        Widget, Pos2, Vec2, Order, Align2,
        Button, Align, Layout, vec2,
        context_menu::{
            SubMenu,
            MenuState,
        },
    },
    epi,
};
#[allow(unused)]
use petgraph::graph::DiGraph;
use seqraph::hypergraph::Hypergraph;
use std::sync::{
    Arc,
    RwLock,
};
#[cfg(feature = "persistence")]
use serde::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct App {
    state: AppState,
}
#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct AppState {
    label: String,

    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    // this how you opt-out of serialization of a member
    #[allow(unused)]
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Hypergraph<char>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            graph_file: None,
            graph: Hypergraph::new(),
        }
    }
}
impl AppState {
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
}
impl App {
    fn top_panel(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.state.open_file_dialog();
                    } else if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });
    }
    fn side_panel(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.state.label);
            });
        })
        .response
        .context_menu(|ui, menu_state| self.context_menu(ui, menu_state));
    }
    fn graph(&mut self, ui: &mut Ui, _frame: &mut epi::Frame<'_>) {
        let node1 = egui::Window::new("node 1")
            //.default_height(500.0)
            .default_size((500.0, 600.0))
            .scroll(true)
            //.resizable(true)
            //.default_width(1.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    "This window is resizable and has a scroll area. You can shrink it to any size",
                );
                ui.separator();
                ui.code("Node 1");
            }).unwrap().response;
        let node2 = egui::Window::new("node 2")
            .default_height(100.0)
            .scroll(true)
            .resizable(true)
            //.default_width(1.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    "This window is resizable and has a scroll area. You can shrink it to any size",
                );
                ui.separator();
                ui.code("Node 2");
            }).unwrap().response;
        let pos1 = ((node1.rect.min.to_vec2() + node1.rect.max.to_vec2())/2.0).to_pos2();
        let pos2 = ((node2.rect.min.to_vec2() + node2.rect.max.to_vec2())/2.0).to_pos2();
        let _line = ui.painter().add(egui::Shape::line_segment([pos1, pos2], egui::Stroke::new(1.0, egui::Color32::WHITE)));
    }
    fn central_panel(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.graph(ui, frame);
            egui::warn_if_debug_build(ui);
        })
        .response
        .context_menu(|ui, menu_state| self.context_menu(ui, menu_state));
    }
    fn context_menu(&mut self, ui: &mut Ui, menu_state: &mut MenuState) {
        //let mut state = self.state.clone();
        if ui.button("Open...").clicked() {
            //state.open_file_dialog();
            menu_state.close();
        }
        menu_state.submenu("SubMenu")
            .show(ui, |ui, menu_state| {
                if ui.button("Open...").clicked() {
                    //state.open_file_dialog();
                    menu_state.close();
                }
                menu_state.submenu("SubMenu")
                    .show(ui, |ui, menu_state| {
                        if ui.button("Open...").clicked() {
                            //state.open_file_dialog();
                            menu_state.close();
                        }
                        let _ = ui.button("Item");
                    });
                let _ = ui.button("Item");
            });
        menu_state.submenu("SubMenu")
            .show(ui, |ui, _menu_state| {
                let _ = ui.button("Item1");
                let _ = ui.button("Item2");
                let _ = ui.button("Item3");
                let _ = ui.button("Item4");
            });
        let _ = ui.button("Item");
        //self.state = state;
    }
}
impl epi::App for App {
    fn name(&self) -> &str {
        "egui template"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, storage: Option<&dyn epi::Storage>) {
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
        self.side_panel(ctx, frame);
        self.central_panel(ctx, frame);
    }
}
use std::future::Future;
#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    tokio::spawn(f);
}
#[allow(unused)]
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
