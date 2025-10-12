use std::{convert::TryFrom, fmt::Debug, sync::RwLockWriteGuard};

use crate::{
    insert::result::ResultExtraction,
    interval::{IntervalGraph, init::InitInterval},
    join::context::frontier::FrontierSplitIterator,
};
use context_search::*;
use context_trace::*;
use std::sync::RwLockReadGuard;

use crate::insert::result::InsertResult;

#[derive(Debug, Clone, Default)]
pub struct InsertTraversal;

impl TraversalKind for InsertTraversal {
    type Trav = HypergraphRef;
    type Container = BftQueue;
    type Policy = AncestorPolicy<Self::Trav>;
}

#[derive(Debug)]
pub struct InsertCtx<R: InsertResult = Child> {
    graph: HypergraphRef,
    _ty: std::marker::PhantomData<R>,
}
impl<R: InsertResult> From<HypergraphRef> for InsertCtx<R> {
    fn from(graph: HypergraphRef) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
}

impl<R: InsertResult> InsertCtx<R> {
    pub fn insert(
        &mut self,
        foldable: impl Foldable,
    ) -> Result<R, ErrorState> {
        self.insert_result(foldable)
            .and_then(|res| res.map_err(|root| root.into()))
    }
    pub fn insert_init(
        &mut self,
        ext: R::Extract,
        init: InitInterval,
    ) -> R {
        let interval = IntervalGraph::from((&mut self.graph.graph_mut(), init));
        let mut ctx =
            FrontierSplitIterator::from((self.graph.clone(), interval));
        let joined = ctx.find_map(|joined| joined).unwrap();
        R::build_with_extract(joined, ext)
    }
    fn insert_result(
        &mut self,
        foldable: impl Foldable,
    ) -> Result<Result<R, R::Error>, ErrorState> {
        match foldable.fold::<InsertTraversal>(self.graph.clone()) {
            Ok(result) => Ok(match CompleteState::try_from(result) {
                Ok(state) => R::try_init(state.root),
                Err(state) => Ok(self.insert_init(
                    <R::Extract as ResultExtraction>::extract_from(&state),
                    InitInterval::from(state),
                )),
            }),
            Err(err) => Err(err),
        }
    }
    pub fn insert_or_get_complete(
        &mut self,
        foldable: impl Foldable,
    ) -> Result<Result<R, R::Error>, ErrorReason> {
        self.insert_result(foldable).map_err(|err| err.reason)
    }
}
impl_has_graph! {
    impl<R: InsertResult> for InsertCtx<R>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_has_graph_mut! {
    impl<R: InsertResult> for InsertCtx<R>,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
