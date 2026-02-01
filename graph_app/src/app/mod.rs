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
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,

    /// Currently selected tab in the central panel
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) selected_tab: CentralTab,

    /// Whether the settings window is open
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) settings_open: bool,

    /// Whether the left panel is open
    pub(crate) left_panel_open: bool,

    /// Whether the right panel is open
    pub(crate) right_panel_open: bool,

    /// Whether the bottom panel is open
    pub(crate) bottom_panel_open: bool,

    /// Currently selected algorithm
    pub(crate) selected_algorithm: Algorithm,

    pub(crate) read_task: Option<tokio::task::JoinHandle<()>>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) cancellation_token: Option<CancellationToken>,

    #[deref]
    #[deref_mut]
    pub(crate) read_ctx: Arc<RwLock<ReadCtx>>,

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
    #[allow(unused)]
    pub fn ctx(&self) -> Option<RwLockReadGuard<'_, ReadCtx>> {
        self.read_ctx.try_read()
    }

    #[allow(unused)]
    pub fn ctx_mut(&mut self) -> Option<RwLockWriteGuard<'_, ReadCtx>> {
        self.read_ctx.try_write()
    }

    #[allow(unused)]
    pub fn vis(&self) -> Option<SyncRwLockReadGuard<'_, GraphVis>> {
        self.vis.read().ok()
    }

    pub fn vis_mut(&self) -> Option<SyncRwLockWriteGuard<'_, GraphVis>> {
        self.vis.write().ok()
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
        self.top_panel(ctx, frame);

        if self.bottom_panel_open {
            self.bottom_panel(ctx);
        }

        // Side panels use show_animated which handles open/closed state internally
        self.left_panel(ctx);
        self.right_panel(ctx);

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
