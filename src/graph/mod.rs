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
use async_std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
pub mod vis;
use vis::GraphVis;
pub use vis::Layout;

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
        new.vis_mut().unwrap().set_graph(g);
        new
    }
    pub fn new() -> Self {
        let graph = Hypergraph::default();
        Self::new_from_graph(graph)
    }
    pub(crate) async fn read(&self) -> RwLockReadGuard<'_, Hypergraph<char>> {
        self.graph.read().await
    }
    pub(crate) async fn write(&self) -> RwLockWriteGuard<'_, Hypergraph<char>> {
        self.graph.write().await
    }
    pub(crate) fn try_read(&self) -> Option<RwLockReadGuard<'_, Hypergraph<char>>> {
        self.graph.try_read()
    }
    #[allow(unused)]
    pub(crate) fn vis(&self) -> Option<RwLockReadGuard<'_, GraphVis>> {
        self.vis.try_read()
    }
    pub(crate) fn vis_mut(&self) -> Option<RwLockWriteGuard<'_, GraphVis>> {
        self.vis.try_write()
    }
    pub fn set_graph(&self, graph: Hypergraph<char>) {
        tokio::runtime::Handle::current().block_on(async {
            *self.write().await = graph;
        });
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn read_text(&mut self, text: impl ToString, ctx: &egui::Context) {
        let text = text.to_string();
        let ctx = ctx.clone();
        let mut graph = self.graph.clone();
        tokio::spawn(async move {
            graph.read_sequence(text.chars()).await;
            println!("done");
            ctx.request_repaint();
        });
    }
    pub fn poll_events(&self) -> Vec<tracing_egui::LogEvent> {
        //println!("polling..");
        tracing_egui::poll_events()
            .drain(..)
            .collect()
    }
    pub fn show(&self, ui: &mut Ui) {
        //println!("got events");
        if let Some(mut vis) = self.vis_mut() {
            let events = self.poll_events();
            if !events.is_empty() {
                vis.update();
            }
            vis.show(ui);
        }
    }
}