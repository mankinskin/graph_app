use std::{
    borrow::Borrow,
    sync::RwLockWriteGuard,
};

use tracing::{
    debug,
    instrument,
};

use crate::{
    insert::{
        context::InsertContext,
        Inserting,
    },
    read::sequence::{
        SequenceIter,
        ToNewTokenIndices,
    },
};
use hypercontext_api::{
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
    path::{
        mutators::move_path::Advance,
        structs::{
            query_range_path::FoldablePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::PatternEndPath,
            },
        },
    },
    traversal::{
        fold::{
            ErrorState,
            Foldable,
        },
        result::FoundRange,
        traversable::{
            impl_traversable,
            impl_traversable_mut,
            Traversable,
            TraversableMut,
        },
    },
};

use super::overlap::{
    chain::OverlapChain,
    iterator::ExpansionIterator,
};
pub trait HasReadContext: Inserting {
    fn read_context<'g>(&'g mut self) -> ReadContext;
    fn read_sequence(
        &mut self,
        //sequence: impl IntoIterator<Item = DefaultToken> + std::fmt::Debug + Send + Sync,
        sequence: impl ToNewTokenIndices,
    ) -> Option<Child> {
        self.read_context().read_sequence(sequence)
    }
    fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child> {
        self.read_context().read_pattern(pattern)
    }
}

impl HasReadContext for ReadContext {
    fn read_context(&mut self) -> ReadContext {
        self.clone()
    }
}
impl<T: HasReadContext> HasReadContext for &'_ mut T {
    fn read_context(&mut self) -> ReadContext {
        (**self).read_context()
    }
}
impl HasReadContext for HypergraphRef {
    fn read_context(&mut self) -> ReadContext {
        //ReadContext::new(self.graph_mut())
        ReadContext::new(self.clone())
    }
}
#[derive(Debug, Clone)]
pub struct ReadContext {
    //pub graph: RwLockWriteGuard<'g, Hypergraph>,
    pub graph: HypergraphRef,
    pub root: Option<Child>,
}

impl ReadContext {
    pub fn new(graph: HypergraphRef) -> Self {
        Self { graph, root: None }
    }
    #[instrument(skip(self, first, cursor))]
    pub fn expand_block(
        &mut self,
        first: Child,
        cursor: &mut PatternEndPath,
    ) -> Child {
        ExpansionIterator::new(self.clone(), cursor, OverlapChain::new(first)).collect()
    }
    #[instrument(skip(self, sequence))]
    pub fn read(
        &mut self,
        mut sequence: PatternEndPath,
    ) {
        //println!("reading known bands");
        while let Some(next) = self.next_known_index(&mut sequence) {
            //println!("found next {:?}", next);
            let next = self.expand_block(next, &mut sequence);
            self.append_index(next);
        }
    }
    #[instrument(skip(self, context))]
    fn next_known_index(
        &mut self,
        context: &mut PatternEndPath,
    ) -> Option<Child> {
        match self.read_one(context.clone()) {
            Ok((index, advanced)) => {
                *context = PatternEndPath::from(advanced);
                Some(index)
            }
            Err(_) => {
                context.advance(self);
                None
            }
        }
    }
    pub fn read_one(
        &mut self,
        query: impl Foldable,
    ) -> Result<(Child, PatternRangePath), ErrorReason> {
        let mut ctx = self.insert_context();
        match ctx.insert(query) {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(c),
                found: Some(FoundRange::Complete(_, p)),
            }) => Ok((c, p)),
            Err(err) => Err(err.reason),
            Ok(v) => Ok(v),
        }
    }
    #[instrument(skip(self))]
    pub fn read_sequence<S: ToNewTokenIndices>(
        &mut self,
        sequence: S,
    ) -> Option<Child> {
        debug!("start reading: {:?}", sequence);
        let sequence = sequence.to_new_token_indices(self);
        let mut sequence = SequenceIter::new(&sequence);
        while let Some((unknown, known)) = sequence.next_block() {
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
        match PatternEndPath::new_directed::<Right, _>(known.borrow()) {
            Ok(path) => self.band_context().read(path),
            Err((err, _)) => match err {
                ErrorReason::SingleIndex(c) => {
                    self.append_index(c);
                    Ok(())
                }
                ErrorReason::EmptyPatterns => Ok(()),
                err => Err(err),
            }
            .unwrap(),
        }
    }
    pub fn band_context(&self) -> ReadContext {
        ReadContext::new(self.graph.clone())
    }
    pub fn insert_context(&self) -> InsertContext {
        InsertContext::new(self.graph.clone())
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
    //pub fn contexter<Side: SplitSide<D>>(&self) -> Contexter<Side> {
    //    Contexter::new(self.insert_context())
    //}
    //pub fn splitter<Side: SplitSide<D>>(&self) -> Splitter<Side> {
    //    Splitter::new(self.insert_context())
    //}
    //fn append_next(&mut self, end_bound: usize, index: Child) -> usize {
    //    self.append_index(index);
    //    0
    //}
}

impl Inserting for ReadContext {
    fn insert_context(&self) -> InsertContext {
        InsertContext::new(self.graph.clone())
    }
}

impl_traversable! {
    impl for ReadContext,
    //Self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
//impl_traversable! {
//    impl for &'_ ReadContext,
//    //self => self.graph.graph();
//    //<'a> &'a Hypergraph
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph>
//}
//impl_traversable! {
//    impl for &'_ mut ReadContext,
//    //self => self.graph.graph();
//    //<'a> &'a Hypergraph
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph>
//}
impl_traversable_mut! {
    impl for ReadContext,
    //self => self.graph.graph_mut();
    //<'a> &'a mut Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
//impl_traversable_mut! {
//    impl for &'_ mut ReadContext,
//    //self => self.graph.graph_mut();
//    //<'a> &'a mut Hypergraph
//    self => self.graph.write().unwrap();
//    <'a> RwLockWriteGuard<'a, Hypergraph>
//}
