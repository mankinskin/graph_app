//! Application state and core functionality.

mod central;
mod menus;
mod panels;
#[cfg(not(target_arch = "wasm32"))]
mod tasks;
#[cfg(target_arch = "wasm32")]
mod tasks_wasm;

use eframe::egui;
#[cfg(feature = "persistence")]
use serde::*;

#[allow(unused)]
use crate::graph::*;
use crate::{
    algorithm::Algorithm,
    graph::Graph,
    output::OutputBuffer,
    read::ReadCtx,
    vis::graph::GraphVis,
    widgets::EditableLabelState,
};
#[cfg(not(target_arch = "wasm32"))]
use async_std::sync::RwLock as AsyncRwLock;
use context_trace::graph::vertex::key::VertexKey;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::AtomicBool;
use std::sync::{
    Arc,
    RwLock as SyncRwLock,
    RwLockReadGuard as SyncRwLockReadGuard,
    RwLockWriteGuard as SyncRwLockWriteGuard,
};
#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

/// A single graph tab with its own graph state
#[derive(Debug)]
pub struct GraphTab {
    pub id: usize,
    pub name: String,
    pub label_state: EditableLabelState,
    pub vis: Arc<SyncRwLock<GraphVis>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub read_ctx: Arc<AsyncRwLock<ReadCtx>>,
    #[cfg(target_arch = "wasm32")]
    pub read_ctx: Arc<SyncRwLock<ReadCtx>>,
    /// Currently selected node in the graph
    pub selected_node: Option<VertexKey>,
}

impl GraphTab {
    pub fn new(
        id: usize,
        name: impl Into<String>,
    ) -> Self {
        let graph = Graph::default();
        Self {
            id,
            name: name.into(),
            label_state: EditableLabelState::default(),
            vis: Arc::new(SyncRwLock::new(GraphVis::new(graph.clone()))),
            #[cfg(not(target_arch = "wasm32"))]
            read_ctx: Arc::new(AsyncRwLock::new(ReadCtx::new(graph))),
            #[cfg(target_arch = "wasm32")]
            read_ctx: Arc::new(SyncRwLock::new(ReadCtx::new(graph))),
            selected_node: None,
        }
    }

    pub fn vis(&self) -> Option<SyncRwLockReadGuard<'_, GraphVis>> {
        self.vis.read().ok()
    }

    pub fn vis_mut(&self) -> Option<SyncRwLockWriteGuard<'_, GraphVis>> {
        self.vis.write().ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn ctx(&self) -> Option<async_std::sync::RwLockReadGuard<'_, ReadCtx>> {
        self.read_ctx.try_read()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn ctx(&self) -> Option<SyncRwLockReadGuard<'_, ReadCtx>> {
        self.read_ctx.read().ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn ctx_mut(
        &self
    ) -> Option<async_std::sync::RwLockWriteGuard<'_, ReadCtx>> {
        self.read_ctx.try_write()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn ctx_mut(&self) -> Option<SyncRwLockWriteGuard<'_, ReadCtx>> {
        self.read_ctx.write().ok()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,

    /// List of open graph tabs
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) tabs: Vec<GraphTab>,

    /// Currently selected tab index
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) selected_tab_id: usize,

    /// Counter for generating unique tab IDs
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) next_tab_id: usize,

    /// Whether the inserter window is open
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) inserter_open: bool,

    /// Whether the inserter window has been manually moved
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) inserter_manually_moved: bool,

    /// Whether the settings window is open
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) settings_open: bool,

    /// Whether the left panel is open
    pub(crate) left_panel_open: bool,

    /// Whether the right panel is open
    pub(crate) right_panel_open: bool,

    /// Whether the bottom panel is open
    pub(crate) bottom_panel_open: bool,

    /// Whether the bottom panel overlaps the left sidebar
    pub(crate) bottom_panel_overlaps_left: bool,

    /// Whether the bottom panel overlaps the right sidebar
    pub(crate) bottom_panel_overlaps_right: bool,

    /// Whether the status bar is visible
    pub(crate) status_bar_open: bool,

    /// Currently selected algorithm
    pub(crate) selected_algorithm: Algorithm,

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) read_task: Option<tokio::task::JoinHandle<()>>,

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) cancellation_token: Option<CancellationToken>,

    /// Wasm: Whether an algorithm is currently running
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) is_running: bool,

    /// Wasm: Cancellation flag for the current operation
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) cancelled: Option<Arc<AtomicBool>>,

    /// Wasm: Flag to track if async task is still running
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) running_flag: Arc<AtomicBool>,

    /// Output buffer for the bottom panel
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) output: OutputBuffer,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let initial_tab = GraphTab::new(0, "Graph 1");
        Self {
            graph_file: None,
            tabs: vec![initial_tab],
            selected_tab_id: 0,
            next_tab_id: 1,
            inserter_open: true,
            inserter_manually_moved: false,
            settings_open: false,
            left_panel_open: true,
            right_panel_open: true,
            bottom_panel_open: true,
            bottom_panel_overlaps_left: false,
            bottom_panel_overlaps_right: false,
            status_bar_open: true,
            selected_algorithm: Algorithm::default(),
            #[cfg(not(target_arch = "wasm32"))]
            read_task: None,
            #[cfg(not(target_arch = "wasm32"))]
            cancellation_token: None,
            #[cfg(target_arch = "wasm32")]
            is_running: false,
            #[cfg(target_arch = "wasm32")]
            cancelled: None,
            #[cfg(target_arch = "wasm32")]
            running_flag: Arc::new(AtomicBool::new(false)),
            output: OutputBuffer::new(),
        }
    }

    /// Get the currently selected tab
    pub fn current_tab(&self) -> Option<&GraphTab> {
        self.tabs.iter().find(|t| t.id == self.selected_tab_id)
    }

    /// Get mutable reference to the currently selected tab
    pub fn current_tab_mut(&mut self) -> Option<&mut GraphTab> {
        self.tabs.iter_mut().find(|t| t.id == self.selected_tab_id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    pub fn ctx(&self) -> Option<async_std::sync::RwLockReadGuard<'_, ReadCtx>> {
        self.current_tab()?.ctx()
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(unused)]
    pub fn ctx(&self) -> Option<SyncRwLockReadGuard<'_, ReadCtx>> {
        self.current_tab()?.ctx()
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    pub fn ctx_mut(
        &self
    ) -> Option<async_std::sync::RwLockWriteGuard<'_, ReadCtx>> {
        self.current_tab()?.ctx_mut()
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(unused)]
    pub fn ctx_mut(&self) -> Option<SyncRwLockWriteGuard<'_, ReadCtx>> {
        self.current_tab()?.ctx_mut()
    }

    #[allow(unused)]
    pub fn vis(&self) -> Option<SyncRwLockReadGuard<'_, GraphVis>> {
        self.current_tab()?.vis()
    }

    pub fn vis_mut(&self) -> Option<SyncRwLockWriteGuard<'_, GraphVis>> {
        self.current_tab()?.vis_mut()
    }

    #[allow(unused)]
    fn open_file_dialog(&mut self) {
        let mut dialog = rfd::AsyncFileDialog::default();
        let res = std::env::current_dir();
        if let Ok(current_dir) = &res {
            dialog = dialog.set_directory(current_dir);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    async fn read_graph_file(
        graph: Arc<AsyncRwLock<Option<String>>>,
        file: &rfd::FileHandle,
    ) {
        let content = file.read().await;
        match std::str::from_utf8(&content[..]) {
            Ok(_content) => {},
            Err(err) => {
                rfd::AsyncMessageDialog::default()
                    .set_description(format!("{}", err))
                    .show()
                    .await;
            },
        }
    }
}

impl eframe::App for App {
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
        // The order determines which panels "own" the space - earlier panels get priority
        self.top_panel(ctx, frame);

        // Status bar (always at very bottom)
        if self.status_bar_open {
            self.status_bar(ctx);
        }

        // Determine panel rendering order based on overlap settings
        // If bottom panel should NOT overlap a side panel, render that side panel first
        let both_overlap =
            self.bottom_panel_overlaps_left && self.bottom_panel_overlaps_right;
        let neither_overlap = !self.bottom_panel_overlaps_left
            && !self.bottom_panel_overlaps_right;

        if both_overlap {
            // Bottom panel first, then both side panels
            self.bottom_panel(ctx);
            self.left_panel(ctx);
            self.right_panel(ctx);
        } else if neither_overlap {
            // Both side panels first, then bottom panel
            self.left_panel(ctx);
            self.right_panel(ctx);
            self.bottom_panel(ctx);
        } else if !self.bottom_panel_overlaps_left {
            // Left first, then bottom, then right
            self.left_panel(ctx);
            self.bottom_panel(ctx);
            self.right_panel(ctx);
        } else {
            // Right first, then bottom, then left
            self.right_panel(ctx);
            self.bottom_panel(ctx);
            self.left_panel(ctx);
        }

        self.central_panel(ctx);

        // Settings window
        self.show_settings_window(ctx);

        // Handle finished tasks (native only)
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_finished_tasks();

        // Handle finished tasks (wasm)
        #[cfg(target_arch = "wasm32")]
        self.poll_finished_tasks();
    }

    fn on_exit(
        &mut self,
        _ctx: Option<&eframe::glow::Context>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        self.abort();
        #[cfg(target_arch = "wasm32")]
        self.abort();
    }
}

#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    tokio::spawn(f);
}

#[allow(unused)]
#[cfg(target_arch = "wasm32")]
fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
