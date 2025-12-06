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
    pub graph: HypergraphRef,
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
            graph,
            insert_texts: vec![String::from("aabbaabbaa")],
            labels: Default::default(),
            rec: None,
        }
    }
}
impl From<rerun::RecordingStream> for Graph {
    fn from(rec: rerun::RecordingStream) -> Self {
        Self {
            graph: Hypergraph::default().into(),
            insert_texts: vec![String::from("aabbaabbaa")],
            labels: Default::default(),
            rec: Some(rec),
        }
    }
}
impl Graph {
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, Hypergraph>> {
        self.graph.read().ok()
    }
    pub fn read(&self) -> RwLockReadGuard<'_, Hypergraph> {
        self.try_read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<'_, Hypergraph> {
        self.graph.write().unwrap()
    }
    pub fn set_graph(
        &self,
        graph: Hypergraph,
    ) {
        *self.write() = graph;
    }
    pub fn clear(&mut self) {
        *self = Self::default();
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
