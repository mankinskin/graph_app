use std::{
    iter::FromIterator,
    sync::RwLockWriteGuard,
};

use rand::seq;
use serde_json::de::Read;
use tracing::{
    debug,
    instrument,
};

use crate::{
    insert::{
        context::InsertContext,
        ToInsertContext,
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
        self,
        fold::foldable::{
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
pub trait HasReadContext: ToInsertContext {
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
    pub graph: HypergraphRef,
    pub sequence: PatternEndPath,
    //pub root: Option<Child>,
}

impl Iterator for ReadContext {
    type Item = Child;
    fn next(&mut self) -> Option<Self::Item> {
        self.read_step()
    }
}
pub enum ReadState {
    Continue(Child, PatternEndPath),
    Stop(PatternEndPath),
}
impl ReadContext {
    pub fn new(
        graph: HypergraphRef,
        sequence: PatternEndPath,
    ) -> Self {
        Self { graph, sequence }
    }
    pub fn expand_block(
        &mut self,
        first: Child,
    ) -> Child {
        ExpansionIterator::new(self.clone(), &mut self.sequence, OverlapChain::new(first)).collect()
    }
    pub fn read_step(&mut self) -> Option<Child> {
        self.next_index().map(|next| self.expand_block(next))
    }
    pub fn read(&mut self) {
        if let Some(next) = self.read_step() {
            self.append_index(next);
            self.read()
        }
    }
    pub fn next_index(&mut self) -> Option<Child> {
        match self.insert_or_get_complete(self.sequence.clone()) {
            Ok((index, advanced)) => {
                self.sequence = PatternEndPath::from(advanced);
                Some(index)
            }
            Err(_) => {
                self.sequence.advance(&self.graph);
                None
            }
        }
    }
    //#[instrument(skip(self))]
    //pub fn read_sequence<S: ToNewTokenIndices>(
    //    &mut self,
    //    sequence: S,
    //) -> Option<Child> {
    //    debug!("start reading: {:?}", sequence);
    //    let sequence = sequence.to_new_token_indices(self);
    //    let mut sequence = SequenceIter::new(&sequence);
    //    while let Some((unknown, known)) = sequence.next_block() {
    //        // todo: read to result type
    //        self.append_pattern(unknown);
    //        self.read_known(known)
    //    }
    //    //println!("reading result: {:?}", index);
    //    self.root
    //}
    //pub fn read_pattern(
    //    &mut self,
    //    known: impl IntoPattern,
    //) -> Option<Child> {
    //    self.read_known(known.into_pattern());
    //    self.root
    //}
    //#[instrument(skip(self, known))]
    //pub fn read_known(
    //    &mut self,
    //    known: Pattern,
    //) {
    //    match PatternEndPath::new_directed::<Right>(known) {
    //        Ok(path) => self.band_context().read(path),
    //        Err((err, _)) => match err {
    //            ErrorReason::SingleIndex(c) => {
    //                self.append_index(c);
    //                Ok(())
    //            }
    //            ErrorReason::EmptyPatterns => Ok(()),
    //            err => Err(err),
    //        }
    //        .unwrap(),
    //    }
    //}
    //pub fn insert_context(&self) -> InsertContext {
    //    InsertContext::from(self.graph.clone())
    //}
    //#[instrument(skip(self, index))]
    //pub fn append_index(
    //    &mut self,
    //    index: impl ToChild,
    //) {
    //    let index = index.to_child();
    //    if let Some(root) = &mut self.root {
    //        let mut graph = self.graph.graph_mut();
    //        let vertex = (*root).vertex_mut(&mut graph);
    //        *root = if index.vertex_index() != root.vertex_index()
    //            && vertex.children.len() == 1
    //            && vertex.parents.is_empty()
    //        {
    //            let (&pid, _) = vertex.expect_any_child_pattern();
    //            graph.append_to_pattern(*root, pid, index)
    //        } else {
    //            graph.insert_pattern(vec![*root, index])
    //        };
    //    } else {
    //        self.root = Some(index);
    //    }
    //}
    ///// append a pattern of new token indices
    ///// returns index of possible new index
    //fn append_pattern(
    //    &mut self,
    //    new: Pattern,
    //) {
    //    match new.len() {
    //        0 => {}
    //        1 => {
    //            let new = new.iter().next().unwrap();
    //            self.append_index(new)
    //        }
    //        _ => {
    //            if let Some(root) = &mut self.root {
    //                let mut graph = self.graph.graph_mut();
    //                let vertex = (*root).vertex_mut(&mut graph);
    //                *root = if vertex.children.len() == 1 && vertex.parents.is_empty() {
    //                    let (&pid, _) = vertex.expect_any_child_pattern();
    //                    graph.append_to_pattern(*root, pid, new)
    //                } else {
    //                    // some old overlaps though
    //                    let new = new.into_pattern();
    //                    graph.insert_pattern([&[*root], new.as_slice()].concat())
    //                };
    //            } else {
    //                let c = self.graph_mut().insert_pattern(new);
    //                self.root = Some(c);
    //            }
    //        }
    //    }
    //}
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

impl ToInsertContext for ReadContext {
    fn insert_context(&self) -> InsertContext {
        InsertContext::from(self.graph.clone())
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
