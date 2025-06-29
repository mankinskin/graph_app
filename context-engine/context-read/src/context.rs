use std::sync::RwLockWriteGuard;

use context_insert::insert::{
    context::InsertCtx,
    result::InsertResult,
    ToInsertCtx,
};
use context_trace::{
    direction::Right,
    graph::{
        getters::ErrorReason,
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
    path::structs::{
        query_range_path::FoldablePath,
        rooted::role_path::PatternEndPath,
    },
    trace::has_graph::HasGraphMut,
};
use tracing::instrument;

use crate::{
    expansion::ExpansionIterator,
    sequence::{
        NextBlock,
        SequenceIter,
        ToNewTokenIndices,
    },
};
pub trait HasReadCtx {
    fn read_context<'g>(&'g mut self) -> ReadCtx;
    fn read_sequence(&mut self) -> Option<Child> {
        self.read_context().read_sequence()
    }
    fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child> {
        self.read_context().read_pattern(pattern)
    }
}

impl HasReadCtx for ReadCtx {
    fn read_context(&mut self) -> ReadCtx {
        self.clone()
    }
}
impl<T: HasReadCtx> HasReadCtx for &'_ mut T {
    fn read_context(&mut self) -> ReadCtx {
        (**self).read_context()
    }
}
impl<S: ToNewTokenIndices + Clone> HasReadCtx for (HypergraphRef, S) {
    fn read_context(&mut self) -> ReadCtx {
        let (graph, seq) = self;
        ReadCtx::new(graph.clone(), seq.clone())
    }
}
impl<S: ToNewTokenIndices + Clone> HasReadCtx for (&mut HypergraphRef, S) {
    fn read_context(&mut self) -> ReadCtx {
        let (graph, seq) = self;
        ReadCtx::new(graph.clone(), seq.clone())
    }
}
#[derive(Debug, Clone)]
pub struct ReadCtx {
    pub graph: HypergraphRef,
    pub root: Option<Child>,
    pub sequence: SequenceIter,
}
pub enum ReadState {
    Continue(Child, PatternEndPath),
    Stop(PatternEndPath),
}
impl Iterator for ReadCtx {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        self.sequence.next().map(|block| self.append_block(block))
    }
}
impl ReadCtx {
    pub fn new(
        mut graph: HypergraphRef,
        seq: impl ToNewTokenIndices,
    ) -> Self {
        let new_indices = seq.to_new_token_indices(&mut graph.graph_mut());
        Self {
            graph,
            sequence: SequenceIter::new(new_indices),
            root: None,
        }
    }
    #[instrument(skip(self))]
    pub fn read_sequence(&mut self) -> Option<Child> {
        self.find_map(|_| None as Option<()>);
        self.root
    }
    #[instrument(skip(self, known))]
    pub fn read_known(
        &mut self,
        known: Pattern,
    ) -> Pattern {
        match PatternEndPath::new_directed::<Right>(known.clone()) {
            Ok(path) => {
                let mut cursor = path.into_range(0);
                let block = ExpansionIterator::new(self.clone(), &mut cursor)
                    .find_largest_bundle()
                    .wrap_into_child(&mut self.graph);
                assert!(cursor.end_path().is_empty());
                [&[block], &cursor.root[cursor.end.root_entry + 1..]].concat()
            },
            Err((err, _)) => match err {
                ErrorReason::SingleIndex(c) => vec![c],
                _ => known,
            },
        }
    }
    fn append_block(
        &mut self,
        block: NextBlock,
    ) {
        let NextBlock { unknown, known } = block;
        // todo: read to result type
        self.append_pattern(unknown);
        let minified = self.read_known(known);
        self.append_pattern(minified);
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    fn append_pattern(
        &mut self,
        new: Pattern,
    ) {
        match new.len() {
            0 => {},
            1 => {
                let new = new.iter().next().unwrap();
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
    //pub fn read_next(&mut self) -> Option<Child> {
    //    match ToInsertCtx::<IndexWithPath>::insert_or_get_complete(
    //        &self.graph,
    //        self.sequence.clone(),
    //    ) {
    //        Ok(IndexWithPath {
    //            index,
    //            path: advanced,
    //        }) => {
    //            self.sequence = advanced;
    //            Some(index)
    //        },
    //        Err(ErrorReason::SingleIndex(index)) => {
    //            self.sequence.advance(&self.graph);
    //            Some(index)
    //        },
    //        Err(_) => {
    //            self.sequence.advance(&self.graph);
    //            None
    //        },
    //    }
    //}
    //pub fn read_pattern(
    //    &mut self,
    //    known: impl IntoPattern,
    //) -> Option<Child> {
    //    self.read_known(known.into_pattern());
    //    self.root
    //}
    //fn append_next(&mut self, end_bound: usize, index: Child) -> usize {
    //    self.append_index(index);
    //    0
    //}
}

impl_has_graph! {
    impl for ReadCtx,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl<R: InsertResult> ToInsertCtx<R> for ReadCtx {
    fn insert_context(&self) -> InsertCtx<R> {
        InsertCtx::from(self.graph.clone())
    }
}
impl_has_graph_mut! {
    impl for ReadCtx,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
