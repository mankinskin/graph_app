use crate::*;
use eframe::egui::{
    self,
    Ui,
};
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};
use seqraph::*;
pub mod vis;
use vis::GraphVis;
pub use vis::Layout;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct Graph {
    pub graph: HypergraphRef<char>,
    pub vis: Arc<RwLock<GraphVis>>,
    pub insert_text: String,
}
impl Graph {
    pub fn new_from_graph(graph: Hypergraph<char>) -> Self {
        Self::new_from_graph_ref(HypergraphRef::from(graph))
    }
    pub fn new_from_graph_ref(graph: HypergraphRef<char>) -> Self {
        let vis = Arc::new(RwLock::new(GraphVis::default()));
        let new = Self {
            graph,
            vis,
            insert_text: String::from("heldldo"),
        };
        let g = new.clone();
        new.vis_mut().set_graph(g);
        new
    }
    pub fn new() -> Self {
        let graph = Hypergraph::default();
        Self::new_from_graph(graph)
    }
    pub(crate) fn try_read(&self) -> Option<RwLockReadGuard<'_, Hypergraph<char>>> {
        self.graph.read().ok()
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, Hypergraph<char>> {
        self.try_read().unwrap()
    }
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, Hypergraph<char>> {
        self.graph.write().unwrap()
    }
    #[allow(unused)]
    pub(crate) fn vis(&self) -> RwLockReadGuard<'_, GraphVis> {
        self.vis.read().unwrap()
    }
    pub(crate) fn vis_mut(&self) -> RwLockWriteGuard<'_, GraphVis> {
        self.vis.write().unwrap()
    }
    pub fn set_graph(&self, graph: Hypergraph<char>) {
        *self.write() = graph;
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn read_text(&mut self, text: impl ToString, ctx: &egui::Context) -> JoinHandle<()> {
        let text = text.to_string();
        let ctx = ctx.clone();
        let mut graph = self.graph.clone();
        tokio::task::spawn_blocking(move || {
            graph.read_sequence(text.chars());
            println!("done reading");
            ctx.request_repaint();
        })
    }
    pub fn poll_events(&self) -> Vec<tracing_egui::LogEvent> {
        //println!("polling..");
        tracing_egui::poll_events()
            .drain(..)
            .collect()
    }
    pub fn show(&self, ui: &mut Ui) {
        //println!("got events");
        let events = self.poll_events();
        let mut vis = self.vis_mut();
        //if !events.is_empty() {
        //}
        vis.update();
        vis.show(ui);
    }
}