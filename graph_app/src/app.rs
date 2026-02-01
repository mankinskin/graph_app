use eframe::egui::{
    self,
    Ui,
};
#[cfg(feature = "persistence")]
use serde::*;

#[allow(unused)]
use crate::graph::*;
use crate::{
    algorithm::Algorithm,
    examples::{
        build_graph1,
        build_graph2,
        build_graph3,
    },
    read::ReadCtx,
    vis::graph::GraphVis,
};
use async_std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
use derive_more::{
    Deref,
    DerefMut,
};
use std::{
    future::Future,
    sync::{
        RwLock as SyncRwLock,
        RwLockReadGuard as SyncRwLockReadGuard,
        RwLockWriteGuard as SyncRwLockWriteGuard,
    },
};
use strum::IntoEnumIterator;
use tokio_util::sync::CancellationToken;

/// Tabs available in the central panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CentralTab {
    #[default]
    Graph,
    Inserter,
}

#[derive(Deref, DerefMut, Debug)]
#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,

    /// Currently selected tab in the central panel
    #[cfg_attr(feature = "persistence", serde(skip))]
    selected_tab: CentralTab,

    /// Whether the settings window is open
    #[cfg_attr(feature = "persistence", serde(skip))]
    settings_open: bool,

    /// Whether the left panel is open
    left_panel_open: bool,

    /// Whether the right panel is open
    right_panel_open: bool,

    /// Whether the bottom panel is open
    bottom_panel_open: bool,

    /// Currently selected algorithm
    selected_algorithm: Algorithm,

    read_task: Option<tokio::task::JoinHandle<()>>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    cancellation_token: Option<CancellationToken>,

    #[deref]
    #[deref_mut]
    read_ctx: Arc<RwLock<ReadCtx>>,

    pub vis: Arc<SyncRwLock<GraphVis>>,
}

impl Default for App {
    fn default() -> Self {
        Self::from(Graph::default())
    }
}
impl From<Graph> for App {
    fn from(graph: Graph) -> Self {
        Self {
            graph_file: None,
            selected_tab: CentralTab::default(),
            settings_open: false,
            left_panel_open: true,
            right_panel_open: false,
            bottom_panel_open: true,
            selected_algorithm: Algorithm::default(),
            read_task: None,
            cancellation_token: None,
            vis: Arc::new(SyncRwLock::new(GraphVis::new(graph.clone()))),
            read_ctx: Arc::new(RwLock::new(ReadCtx::new(graph))),
        }
    }
}
impl App {
    //#[allow(unused)]
    //pub fn from_graph_ref(graph: HypergraphRef) -> Self {
    //    Self::from_graph(Graph::new_from_graph_ref(graph))
    //}
    #[allow(unused)]
    pub fn ctx(&self) -> Option<RwLockReadGuard<'_, ReadCtx>> {
        self.read_ctx.try_read()
    }
    #[allow(unused)]
    pub fn ctx_mut(&mut self) -> Option<RwLockWriteGuard<'_, ReadCtx>> {
        self.read_ctx.try_write()
    }
    pub fn context_menu(
        &mut self,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            ui.label("Quick Insert:");
            if let Some(mut ctx) = self.ctx_mut() {
                for text in &mut ctx.graph_mut().insert_texts {
                    ui.text_edit_singleline(text);
                }
            }
            if ui.button("Go").clicked() {
                self.start_read();
                //self.ctx_mut().graph.insert_text = String::new();
                ui.close();
            }
            if ui.button("Cancel").clicked() {
                self.abort();
            }
        });
        if ui.button("Open Inserter").clicked() {
            self.selected_tab = CentralTab::Inserter;
            ui.close();
        }
        ui.menu_button("Load preset...", |ui| {
            if let Some(ctx) = self.ctx() {
                if ui.button("Graph 1").clicked() {
                    ctx.graph().set_graph(build_graph1());
                    ui.close();
                }
                if ui.button("Graph 2").clicked() {
                    ctx.graph().set_graph(build_graph2());
                    ui.close();
                }
                if ui.button("Graph 3").clicked() {
                    ctx.graph().set_graph(build_graph3());
                    ui.close();
                }
            }
        });
        if let Some(mut ctx) = self.ctx_mut() {
            if ui.button("Clear").clicked() {
                ctx.graph_mut().clear();
                ui.close();
            }
        }
    }
    #[allow(unused)]
    async fn read_graph_file(
        graph: Arc<RwLock<Option<String>>>,
        file: &rfd::FileHandle,
    ) {
        let content = file.read().await;
        match std::str::from_utf8(&content[..]) {
            Ok(content) => {},
            Err(err) => {
                rfd::AsyncMessageDialog::default()
                    .set_description(format!("{}", err))
                    .show()
                    .await;
            },
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
    fn top_panel(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Edit menu
                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear Graph").clicked() {
                        if let Some(mut ctx) = self.ctx_mut() {
                            ctx.graph_mut().clear();
                        }
                        ui.close();
                    }
                });

                // View menu
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.left_panel_open, "Left Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.right_panel_open, "Right Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.bottom_panel_open, "Bottom Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.separator();
                    ui.label("Central Tab:");
                    if ui
                        .selectable_label(
                            self.selected_tab == CentralTab::Graph,
                            "Graph",
                        )
                        .clicked()
                    {
                        self.selected_tab = CentralTab::Graph;
                        ui.close();
                    }
                    if ui
                        .selectable_label(
                            self.selected_tab == CentralTab::Inserter,
                            "Inserter",
                        )
                        .clicked()
                    {
                        self.selected_tab = CentralTab::Inserter;
                        ui.close();
                    }
                    ui.separator();
                    if ui
                        .checkbox(&mut self.settings_open, "Settings Window")
                        .clicked()
                    {
                        ui.close();
                    }
                });

                // Presets menu
                ui.menu_button("Presets", |ui| {
                    if let Some(ctx) = self.ctx() {
                        if ui.button("Graph 1").clicked() {
                            ctx.graph().set_graph(build_graph1());
                            ui.close();
                        }
                        if ui.button("Graph 2").clicked() {
                            ctx.graph().set_graph(build_graph2());
                            ui.close();
                        }
                        if ui.button("Graph 3").clicked() {
                            ctx.graph().set_graph(build_graph3());
                            ui.close();
                        }
                    }
                });

                // Right-aligned items
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("⚙").on_hover_text("Settings").clicked()
                        {
                            self.settings_open = !self.settings_open;
                        }
                    },
                );
            });
        });
    }

    fn left_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .min_width(150.0)
            .show_animated(ctx, self.left_panel_open, |ui| {
                ui.heading("Tools");
                ui.separator();

                // Algorithm selection
                ui.label("Algorithm:");
                egui::ComboBox::from_id_salt("left_panel_algorithm")
                    .selected_text(self.selected_algorithm.to_string())
                    .show_ui(ui, |ui| {
                        for algorithm in Algorithm::iter() {
                            ui.selectable_value(
                                &mut self.selected_algorithm,
                                algorithm,
                                algorithm.to_string(),
                            );
                        }
                    });

                ui.add_space(10.0);
                ui.label(self.selected_algorithm.description());

                ui.add_space(20.0);
                ui.separator();

                // Insert controls
                ui.heading("Insert");
                if let Some(mut read_ctx) = self.ctx_mut() {
                    for text in &mut read_ctx.graph_mut().insert_texts {
                        ui.text_edit_singleline(text);
                    }
                    if ui.button("+ Add Text").clicked() {
                        read_ctx.graph_mut().insert_texts.push(String::new());
                    }
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("▶ Run").clicked() && self.read_task.is_none()
                    {
                        self.start_read();
                    }
                    if self.read_task.is_some()
                        && ui.button("⏹ Cancel").clicked()
                    {
                        self.abort();
                    }
                });
            });
    }

    fn right_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(200.0)
            .min_width(150.0)
            .show_animated(ctx, self.right_panel_open, |ui| {
                ui.heading("Properties");
                ui.separator();

                // Show graph info
                if let Some(read_ctx) = self.ctx() {
                    let graph = read_ctx.graph();
                    if let Some(graph_ref) = graph.try_read() {
                        ui.label(format!(
                            "Vertices: {}",
                            graph_ref.vertex_count()
                        ));
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.heading("Selection");
                ui.label("No selection");
            });
    }

    fn bottom_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Show task status
                    if self.read_task.is_some() {
                        ui.spinner();
                        ui.label("Processing...");

                        // Show progress if available
                        if let Some(read_ctx) = self.ctx() {
                            if let Some(status) = read_ctx.status() {
                                let status = status.read().unwrap();
                                ui.separator();
                                ui.label(format!("Pass: {:?}", status.pass()));
                                let progress = *status.steps() as f32
                                    / *status.steps_total() as f32;
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .desired_width(150.0)
                                        .show_percentage(),
                                );
                            }
                        }
                    } else {
                        ui.label("Ready");
                    }
                });
            });
    }
    #[allow(unused)]
    pub fn vis(&self) -> Option<SyncRwLockReadGuard<'_, GraphVis>> {
        self.vis.read().ok()
    }
    pub fn vis_mut(&self) -> Option<SyncRwLockWriteGuard<'_, GraphVis>> {
        self.vis.write().ok()
    }
    fn central_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.selected_tab,
                        CentralTab::Graph,
                        "📊 Graph",
                    );
                    ui.selectable_value(
                        &mut self.selected_tab,
                        CentralTab::Inserter,
                        "✏ Inserter",
                    );
                });
                ui.separator();

                // Get viewport rect for constraining windows
                let viewport_rect = ui.available_rect_before_wrap();

                match self.selected_tab {
                    CentralTab::Graph =>
                        if let Some(mut vis) = self.vis_mut() {
                            vis.show(ui)
                        },
                    CentralTab::Inserter => {
                        self.show_inserter_tab(ui, viewport_rect);
                    },
                }

                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }

    fn show_inserter_tab(
        &mut self,
        ui: &mut Ui,
        _viewport_rect: egui::Rect,
    ) {
        ui.heading("Text Inserter");
        ui.add_space(10.0);

        // Show currently selected algorithm
        ui.horizontal(|ui| {
            ui.label("Algorithm:");
            egui::ComboBox::from_id_salt("inserter_algorithm")
                .selected_text(self.selected_algorithm.to_string())
                .show_ui(ui, |ui| {
                    for algorithm in Algorithm::iter() {
                        ui.selectable_value(
                            &mut self.selected_algorithm,
                            algorithm,
                            algorithm.to_string(),
                        );
                    }
                });
        });

        ui.add_space(5.0);
        ui.label(self.selected_algorithm.description());

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label("Input texts:");
        ui.add_space(5.0);

        if let Some(mut read_ctx) = self.ctx_mut() {
            let texts = &mut read_ctx.graph_mut().insert_texts;
            let mut to_remove = None;

            for (i, text) in texts.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(text).desired_width(400.0),
                    );
                    if ui.button("✖").on_hover_text("Remove").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(idx) = to_remove {
                texts.remove(idx);
            }

            ui.add_space(5.0);
            if ui.button("+ Add Text").clicked() {
                texts.push(String::new());
            }
        }

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let is_running = self.read_task.is_some();

            if ui
                .add_enabled(!is_running, egui::Button::new("▶ Run"))
                .clicked()
            {
                self.start_read();
            }

            if is_running {
                if ui.button("⏹ Cancel").clicked() {
                    self.abort();
                }
                ui.spinner();
                ui.label("Processing...");
            }
        });
    }
    fn start_read(&mut self) {
        // Create cancellation token for this operation
        let cancellation_token = CancellationToken::new();
        self.cancellation_token = Some(cancellation_token.clone());

        let ctx = self.read_ctx.clone();
        let algorithm = self.selected_algorithm;
        let task = tokio::spawn(async move {
            let mut ctx = ctx.write().await;
            ctx.run_algorithm(algorithm, cancellation_token).await;
        });
        self.read_task = Some(task);
    }
    fn abort(&mut self) {
        println!("Aborting read operation...");

        // Cancel via the cancellation token first
        if let Some(token) = &self.cancellation_token {
            println!("Cancelling via token...");
            token.cancel();
        }

        // Immediately abort the task - don't wait
        if let Some(handle) = self.read_task.take() {
            println!("Aborting task via handle...");
            handle.abort();
        }

        // Clear the cancellation token
        self.cancellation_token = None;
    }
}
impl eframe::App for App {
    //fn max_size_points(&self) -> egui::Vec2
    //{
    //    (f32::INFINITY, f32::INFINITY).into()
    //}
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(
        &mut self,
        storage: &mut dyn epi::Storage,
    ) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        // Panels must be added in this order: top/bottom first, then sides, then central
        self.top_panel(ctx, frame);

        if self.bottom_panel_open {
            self.bottom_panel(ctx);
        }

        // Side panels use show_animated which handles open/closed state internally
        self.left_panel(ctx);
        self.right_panel(ctx);

        self.central_panel(ctx);

        // Settings window with algorithm dropdown
        if self.settings_open {
            egui::Window::new("Settings")
                .open(&mut self.settings_open)
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Algorithm Selection");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("Algorithm:");
                        egui::ComboBox::from_id_salt("algorithm_selector")
                            .selected_text(self.selected_algorithm.to_string())
                            .show_ui(ui, |ui| {
                                for algorithm in Algorithm::iter() {
                                    ui.selectable_value(
                                        &mut self.selected_algorithm,
                                        algorithm,
                                        algorithm.to_string(),
                                    );
                                }
                            });
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);

                    // Show algorithm description
                    ui.label("Description:");
                    ui.label(self.selected_algorithm.description());
                });
        }

        // Handle finished tasks
        if self
            .read_task
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false)
        {
            let task = self.read_task.take().unwrap();
            // Clear the cancellation token since task is done
            self.cancellation_token = None;
            tokio::runtime::Handle::current().spawn(task);
        }
    }
    fn on_exit(
        &mut self,
        _ctx: Option<&eframe::glow::Context>,
    ) {
        self.abort()
    }
}
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
