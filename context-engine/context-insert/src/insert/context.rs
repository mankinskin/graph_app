use std::{
    convert::TryFrom,
    fmt::Debug,
    sync::RwLockWriteGuard,
};

use crate::{
    insert::result::ResultExtraction,
    interval::{
        IntervalGraph,
        init::InitInterval,
    },
    join::context::frontier::FrontierSplitIterator,
};
use context_search::{
    search::context::AncestorPolicy,
    traversal::{
        TraversalKind,
        container::bft::BftQueue,
        fold::foldable::{
            ErrorState,
            Foldable,
        },
        result::{
            CompleteState,
            FinishedKind,
        },
    },
};
use context_trace::{
    graph::{
        Hypergraph,
        HypergraphRef,
        getters::ErrorReason,
        vertex::child::Child,
    },
    impl_has_graph,
    impl_has_graph_mut,
    trace::has_graph::HasGraphMut,
};
use std::sync::RwLockReadGuard;

use super::result::InsertResult;

#[derive(Debug, Clone, Default)]
pub struct InsertTraversal;

impl TraversalKind for InsertTraversal {
    type Trav = HypergraphRef;
    type Container = BftQueue;
    type Policy = AncestorPolicy<Self::Trav>;
}

#[derive(Debug)]
pub struct InsertContext<R: InsertResult = Child> {
    graph: HypergraphRef,
    _ty: std::marker::PhantomData<R>,
}
impl<R: InsertResult> From<HypergraphRef> for InsertContext<R> {
    fn from(graph: HypergraphRef) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
}

impl<R: InsertResult> InsertContext<R> {
    pub fn insert(
        &mut self,
        foldable: impl Foldable,
    ) -> Result<R, ErrorState> {
        match foldable.fold::<InsertTraversal>(self.graph.clone()) {
            Ok(result) => match CompleteState::try_from(result) {
                Ok(state) => R::try_init(state.root)
                    .map_err(|index| ErrorReason::SingleIndex(index).into()),
                Err(state) => Ok(self.insert_init(
                    <R::Extract as ResultExtraction>::extract_from(&state),
                    InitInterval::from(state),
                )),
            },
            Err(err) => Err(err),
        }
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
    pub fn insert_or_get_complete(
        &mut self,
        query: impl Foldable,
    ) -> Result<R, ErrorReason> {
        match self.insert(query) {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(c),
                found: Some(FinishedKind::Complete(_)),
            }) => R::try_init(c)
                .map_err(|index| ErrorReason::SingleIndex(index).into()),
            Err(err) => Err(err.reason),
            Ok(r) => Ok(r),
        }
    }
}
impl_has_graph! {
    impl<R: InsertResult> for InsertContext<R>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_has_graph_mut! {
    impl<R: InsertResult> for InsertContext<R>,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
