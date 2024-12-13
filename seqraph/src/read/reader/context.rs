use std::{
    borrow::Borrow,
    sync::{
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};

use tracing::{
    debug,
    instrument,
};

use crate::{
    insert::context::InsertContext,
    read::{
        bands::BandsContext,
        sequence::{
            SequenceIter,
            ToNewTokenIndices,
        },
    },
};
use hypercontext_api::{
    direction::Right,
    graph::{
        getters::NoMatch,
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
    traversal::{
        path::structs::query_range_path::PatternPrefixPath,
        traversable::{
            impl_traversable,
            impl_traversable_mut,
            Traversable,
            TraversableMut,
        },
    },
};

#[derive(Debug)]
pub struct ReadContext<'g> {
    //pub graph: RwLockWriteGuard<'g, Hypergraph>,
    pub graph: HypergraphRef,
    pub root: Option<Child>,
    _ty: std::marker::PhantomData<&'g ()>,
}
//impl Deref for ReadContext<'_> {
//    type Target = Hypergraph;
//    fn deref(&self) -> &Self::Target {
//        self.graph.deref()
//    }
//}
//impl DerefMut for ReadContext<'_> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        self.graph.deref_mut()
//    }
//}

impl<'g> ReadContext<'g> {
    //pub fn new(graph: RwLockWriteGuard<'g, Hypergraph>) -> Self {
    pub fn new(graph: HypergraphRef) -> Self {
        Self {
            graph,
            root: None,
            _ty: Default::default(),
        }
    }
    #[instrument(skip(self))]
    pub fn read_sequence<N, S: ToNewTokenIndices<N>>(
        &mut self,
        sequence: S,
    ) -> Option<Child> {
        debug!("start reading: {:?}", sequence);
        let sequence = sequence.to_new_token_indices(self);
        let mut sequence = SequenceIter::new(&sequence);
        while let Some((unknown, known)) = sequence.next_block(self) {
            // todo: read to result type
            self.append_pattern(unknown);
            self.read_known(known)
        }
        //println!("reading result: {:?}", index);
        self.root
    }
    pub fn read_pattern(
        &mut self,
        known: impl IntoPattern,
    ) -> Option<Child> {
        self.read_known(known.into_pattern());
        self.root
    }
    #[instrument(skip(self, known))]
    pub fn read_known(
        &mut self,
        known: Pattern,
    ) {
        match PatternPrefixPath::new_directed::<Right, _>(known.borrow()) {
            Ok(path) => self.bands().read(path),
            Err((err, _)) => match err {
                NoMatch::SingleIndex(c) => {
                    self.append_index(c);
                    Ok(())
                }
                NoMatch::EmptyPatterns => Ok(()),
                err => Err(err),
            }
            .unwrap(),
        }
    }
    pub fn bands(&self) -> BandsContext {
        BandsContext::new(self.graph.clone())
    }
    pub fn indexer(&self) -> InsertContext {
        InsertContext::new(self.graph.clone())
    }
    //pub fn contexter<Side: IndexSide<D>>(&self) -> Contexter<Side> {
    //    Contexter::new(self.indexer())
    //}
    //pub fn splitter<Side: IndexSide<D>>(&self) -> Splitter<Side> {
    //    Splitter::new(self.indexer())
    //}
    //fn append_next(&mut self, end_bound: usize, index: Child) -> usize {
    //    self.append_index(index);
    //    0
    //}
    #[instrument(skip(self, index))]
    fn append_index(
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
                graph.insert_pattern([*root, index])
            };
        } else {
            self.root = Some(index);
        }
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    fn append_pattern(
        &mut self,
        new: impl IntoPattern,
    ) {
        match new.borrow().len() {
            0 => {}
            1 => {
                let new = new.borrow().iter().next().unwrap();
                self.append_index(new)
            }
            _ => {
                if let Some(root) = &mut self.root {
                    let mut graph = self.graph.graph_mut();
                    let vertex = (*root).vertex_mut(&mut graph);
                    *root = if vertex.children.len() == 1 && vertex.parents.is_empty() {
                        let (&pid, _) = vertex.expect_any_child_pattern();
                        graph.append_to_pattern(*root, pid, new)
                    } else {
                        // some old overlaps though
                        let new = new.into_pattern();
                        graph.insert_pattern([&[*root], new.as_slice()].concat())
                    };
                } else {
                    let c = self.graph_mut().insert_pattern(new);
                    self.root = Some(c);
                }
            }
        }
    }
}

impl_traversable! {
    impl for ReadContext<'_>,
    //Self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ ReadContext<'_>,
    //self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ mut ReadContext<'_>,
    //self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for ReadContext<'_>,
    //self => self.graph.graph_mut();
    //<'a> &'a mut Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for &'_ mut ReadContext<'_>,
    //self => self.graph.graph_mut();
    //<'a> &'a mut Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
