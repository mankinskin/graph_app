use std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

use context_trace::{
    graph::{
        vertex::key::VertexKey,
        Hypergraph,
        HypergraphRef,
    },
    HashSet,
};
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};

#[derive(Clone, Debug)]
pub struct Graph {
    /// Wrapped in RwLock to allow replacing the entire graph
    pub graph: Arc<RwLock<HypergraphRef>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub rec: Option<rerun::RecordingStream>,
    pub insert_texts: Vec<String>,
    pub labels: Arc<RwLock<HashSet<VertexKey>>>,
}
impl Default for Graph {
    fn default() -> Self {
        let graph = Hypergraph::default();
        Self::from(graph)
    }
}
impl From<Hypergraph> for Graph {
    fn from(graph: Hypergraph) -> Self {
        Self::from(HypergraphRef::from(graph))
    }
}
impl From<HypergraphRef> for Graph {
    fn from(graph: HypergraphRef) -> Self {
        Self {
            graph: Arc::new(RwLock::new(graph)),
            insert_texts: vec![String::from("aabbaabbaa")],
            labels: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            rec: None,
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl From<rerun::RecordingStream> for Graph {
    fn from(rec: rerun::RecordingStream) -> Self {
        Self {
            graph: Arc::new(RwLock::new(HypergraphRef::from(
                Hypergraph::default(),
            ))),
            insert_texts: vec![String::from("aabbaabbaa")],
            labels: Default::default(),
            rec: Some(rec),
        }
    }
}
impl Graph {
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, HypergraphRef>> {
        self.graph.read().ok()
    }
    pub fn read(&self) -> RwLockReadGuard<'_, HypergraphRef> {
        self.try_read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<'_, HypergraphRef> {
        self.graph.write().unwrap()
    }
    pub fn set_graph(
        &self,
        graph: Hypergraph,
    ) {
        *self.write() = HypergraphRef::from(graph);
    }
    pub fn clear(&self) {
        // Replace the underlying graph with a new empty one, keeping the same Arc
        *self.write() = HypergraphRef::from(Hypergraph::default());
    }
    //pub fn read_text(
    //    &mut self,
    //    text: impl ToString,
    //    ctx: &egui::Context,
    //) -> JoinHandle<()> {
    //    let text = text.to_string();
    //    let ctx = ctx.clone();
    //    let mut graph = self.graph.clone();
    //    tokio::task::spawn_blocking(move || {
    //        graph.read_sequence(text.chars());
    //        println!("done reading");
    //        ctx.request_repaint();
    //    })
    //}
}
