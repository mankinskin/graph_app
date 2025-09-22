use context_trace::graph::HypergraphRef;
use eframe::{
    egui::{
        self,
        ThemePreference,
        Ui,
    },
    CreationContext,
};
use ngrams::graph::{
    traversal::pass::CancelReason,
    Status,
    StatusHandle,
};
#[cfg(feature = "persistence")]
use serde::*;

#[allow(unused)]
use crate::graph::*;
use crate::{
    examples::{
        build_graph1,
        build_graph2,
        build_graph3,
    },
    vis::{
        graph::GraphVis,
        status::ShowStatus,
    },
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
    hash::{
        DefaultHasher,
        Hash,
        Hasher,
    },
    sync::{
        RwLock as SyncRwLock,
        RwLockReadGuard as SyncRwLockReadGuard,
        RwLockWriteGuard as SyncRwLockWriteGuard,
    },
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ReadCtx {
    graph: Graph,
    status: Option<ngrams::graph::StatusHandle>,
}
impl ReadCtx {
    pub async fn read_text(
        &mut self,
        cancellation_token: CancellationToken,
    ) {
        println!("Task running on thread {:?}", std::thread::current().id());

        let graph = self.graph.graph.clone();
        let labels = self.graph.labels.clone();
        let insert_texts = self.graph.insert_texts.clone();

        let status = StatusHandle::from(Status::new(insert_texts.clone()));
        self.status = Some(status.clone());
        //let corpus_name = "7547453137468837744".to_string();
        let corpus_name = {
            let mut hasher = DefaultHasher::new();
            insert_texts.hash(&mut hasher);
            format!("{:?}", hasher.finish())
        };
        let corpus = ngrams::graph::Corpus::new(corpus_name, insert_texts);

        // Parse has periodic cancellation checks during the parse operation
        // Use select to race between parsing and cancellation
        tokio::select! {
            res = ngrams::graph::parse_corpus(
            corpus,
            status,
            cancellation_token.clone(),
        ) => {
                match res {
                    Ok(res) => {
                        self.graph.insert_texts.clear();
                        *graph.write().unwrap() = res.graph;
                        *labels.write().unwrap() = res.labels;
                    },
                    Err(CancelReason::Cancelled) => {
                        println!("Parse operation was cancelled via token");
                    },
                    Err(CancelReason::Error) => {
                        println!("Parse operation panicked");
                    },
                }
            }
            _ = cancellation_token.cancelled() => {
                println!("Parse operation was cancelled via token during execution");
            }
        };

        println!("Task done.");
    }
}

#[derive(Deref, DerefMut, Debug)]
#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    inserter: bool,
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
        Self::from_graph(Graph::default())
    }
}
impl App {
    pub fn new(creation_context: &CreationContext<'_>) -> Self {
        creation_context.egui_ctx.set_theme(ThemePreference::Dark);
        Self {
            ..Default::default()
        }
    }
    #[allow(unused)]
    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph_file: None,
            inserter: true,
            read_task: None,
            cancellation_token: None,
            vis: Arc::new(SyncRwLock::new(GraphVis::new(graph.clone()))),
            read_ctx: Arc::new(RwLock::new(ReadCtx {
                graph,
                status: None,
            })),
        }
    }
    #[allow(unused)]
    pub fn from_graph_ref(graph: HypergraphRef) -> Self {
        Self::from_graph(Graph::new_from_graph_ref(graph))
    }
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
                for text in &mut ctx.graph.insert_texts {
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
            self.inserter = true;
            ui.close();
        }
        ui.menu_button("Load preset...", |ui| {
            if let Some(ctx) = self.ctx() {
                if ui.button("Graph 1").clicked() {
                    ctx.graph.set_graph(build_graph1());
                    ui.close();
                }
                if ui.button("Graph 2").clicked() {
                    ctx.graph.set_graph(build_graph2());
                    ui.close();
                }
                if ui.button("Graph 3").clicked() {
                    ctx.graph.set_graph(build_graph3());
                    ui.close();
                }
            }
        });
        if let Some(mut ctx) = self.ctx_mut() {
            if ui.button("Clear").clicked() {
                ctx.graph.clear();
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Actions...", |ui| self.context_menu(ui));
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                        }
                    },
                )
            })
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
                self.vis_mut().map(|mut vis| vis.show(ui));
                //tracing_egui::show(ui.ctx(), &mut true);
                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }
    fn start_read(&mut self) {
        // Create cancellation token for this operation
        let cancellation_token = CancellationToken::new();
        self.cancellation_token = Some(cancellation_token.clone());

        let ctx = self.read_ctx.clone();
        let task = tokio::spawn(async move {
            let mut ctx = ctx.write().await;
            ctx.read_text(cancellation_token).await;
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
        self.top_panel(ctx, frame);
        self.central_panel(ctx);
        if self.inserter {
            egui::Window::new("Inserter").show(ctx, |ui| {
                if let Some(mut ctx) = self.ctx_mut() {
                    for text in &mut ctx.graph.insert_texts {
                        ui.text_edit_singleline(text);
                    }
                    if ui.button("+").clicked() {
                        ctx.graph.insert_texts.push(String::new());
                    }
                }
                if ui.button("Insert").clicked() && self.read_task.is_none() {
                    self.start_read();
                }
                if self.read_task.is_some() && ui.button("Cancel").clicked() {
                    self.abort()
                }
            });
        }
        if let Some(read_ctx) = self.ctx() {
            if let Some(status) = read_ctx.status.as_ref() {
                ShowStatus(&*status.read().unwrap()).show(ctx);
            }
        }
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
