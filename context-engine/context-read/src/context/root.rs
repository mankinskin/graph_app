use std::sync::RwLockWriteGuard;

use context_insert::insert::{
    context::InsertCtx,
    result::InsertResult,
    ToInsertCtx,
};
use context_trace::{
    graph::{
        vertex::{
            child::Child,
            has_vertex_data::HasVertexDataMut,
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            pattern::{
                IntoPattern,
                Pattern,
            },
        },
        Hypergraph,
        HypergraphRef,
    },
    impl_has_graph,
    impl_has_graph_mut,
    trace::has_graph::HasGraphMut,
};
use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;
use tracing::instrument;

#[derive(Debug, Clone, Deref, DerefMut, new)]
pub struct RootManager {
    #[deref]
    #[deref_mut]
    pub graph: HypergraphRef,
    #[new(default)]
    pub root: Option<Child>,
}

impl RootManager {
    /// append a pattern of new token indices
    /// returns index of possible new index
    pub fn append_pattern(
        &mut self,
        new: Pattern,
    ) {
        match new.len() {
            0 => {},
            1 => {
                let new = new.first().unwrap();
                self.append_index(new)
            },
            _ => {
                if let Some(root) = &mut self.root {
                    let mut graph = self.graph.graph_mut();
                    let vertex = (*root).vertex_mut(&mut graph);
                    *root = if vertex.children.len() == 1
                        && vertex.parents.is_empty()
                    {
                        let (&pid, _) = vertex.expect_any_child_pattern();
                        graph.append_to_pattern(*root, pid, new)
                    } else {
                        // some old overlaps though
                        let new = new.into_pattern();
                        graph
                            .insert_pattern([&[*root], new.as_slice()].concat())
                    };
                } else {
                    let c = self.graph_mut().insert_pattern(new);
                    self.root = Some(c);
                }
            },
        }
    }
    #[instrument(skip(self, index))]
    pub fn append_index(
        &mut self,
        index: impl ToChild,
    ) {
        let index = index.to_child();
        if let Some(root) = &mut self.root {
            let mut graph = self.graph.graph_mut();
            let vertex = (*root).vertex_mut(&mut graph);
            *root = if index.vertex_index() != root.vertex_index()
                && vertex.children.len() == 1
                && vertex.parents.is_empty()
            {
                let (&pid, _) = vertex.expect_any_child_pattern();
                graph.append_to_pattern(*root, pid, index)
            } else {
                graph.insert_pattern(vec![*root, index])
            };
        } else {
            self.root = Some(index);
        }
    }
}

impl_has_graph! {
    impl for RootManager,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl<R: InsertResult> ToInsertCtx<R> for RootManager {
    fn insert_context(&self) -> InsertCtx<R> {
        InsertCtx::from(self.graph.clone())
    }
}
impl_has_graph_mut! {
    impl for RootManager,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
