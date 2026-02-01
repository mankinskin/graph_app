//! Application state and core functionality.

mod central;
mod menus;
mod panels;
mod tasks;

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
use async_std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
use std::{
    future::Future,
    sync::{
        RwLock as SyncRwLock,
        RwLockReadGuard as SyncRwLockReadGuard,
        RwLockWriteGuard as SyncRwLockWriteGuard,
    },
};
use tokio_util::sync::CancellationToken;

/// A single graph tab with its own graph state
#[derive(Debug)]
pub struct GraphTab {
    pub id: usize,
    pub name: String,
    pub label_state: EditableLabelState,
    pub vis: Arc<SyncRwLock<GraphVis>>,
    pub read_ctx: Arc<RwLock<ReadCtx>>,
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
            read_ctx: Arc::new(RwLock::new(ReadCtx::new(graph))),
        }
    }

    pub fn vis(&self) -> Option<SyncRwLockReadGuard<'_, GraphVis>> {
        self.vis.read().ok()
    }

    pub fn vis_mut(&self) -> Option<SyncRwLockWriteGuard<'_, GraphVis>> {
        self.vis.write().ok()
    }

    pub fn ctx(&self) -> Option<RwLockReadGuard<'_, ReadCtx>> {
        self.read_ctx.try_read()
    }

    pub fn ctx_mut(&self) -> Option<RwLockWriteGuard<'_, ReadCtx>> {
        self.read_ctx.try_write()
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

    pub(crate) read_task: Option<tokio::task::JoinHandle<()>>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) cancellation_token: Option<CancellationToken>,

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
            right_panel_open: false,
            bottom_panel_open: true,
            bottom_panel_overlaps_left: false,
            bottom_panel_overlaps_right: false,
            status_bar_open: true,
            selected_algorithm: Algorithm::default(),
            read_task: None,
            cancellation_token: None,
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

    #[allow(unused)]
    pub fn ctx(&self) -> Option<RwLockReadGuard<'_, ReadCtx>> {
        self.current_tab()?.ctx()
    }

    #[allow(unused)]
    pub fn ctx_mut(&self) -> Option<RwLockWriteGuard<'_, ReadCtx>> {
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

    #[allow(unused)]
    async fn read_graph_file(
        graph: Arc<RwLock<Option<String>>>,
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

        // Handle finished tasks
        self.poll_finished_tasks();
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
