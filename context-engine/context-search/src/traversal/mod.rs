use crate::traversal::iterator::r#match::{
    MatchContext,
    TraceNode::Parent,
};
use container::StateContainer;
use context_trace::trace::{
    cache::TraceCache,
    has_graph::{
        HasGraph,
        TravKind,
    },
    traceable::Traceable,
    TraceContext,
};
use derive_new::new;
use fold::foldable::ErrorState;
use iterator::{
    end::EndIterator,
    policy::DirectedTraversalPolicy,
};
use state::{
    end::{
        EndKind,
        EndReason,
        EndState,
        TraceStart,
    },
    start::StartCtx,
};
use std::{
    fmt::Debug,
    ops::ControlFlow,
};
pub mod compare;
pub mod container;
pub mod fold;
pub mod iterator;
pub mod result;
pub mod root_cursor;
pub mod state;

pub trait TraversalKind: Debug + Default {
    type Trav: HasGraph;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}
#[derive(Debug, Clone, Copy)]
pub enum OptGen<Y> {
    Yield(Y),
    Pass,
}

/// context for generating next states
#[derive(Debug, new)]
pub struct TraversalContext<K: TraversalKind> {
    pub end_iter: EndIterator<K>,
    pub last_end: EndState,
}
impl<K: TraversalKind> Unpin for TraversalContext<K> {}

impl<K: TraversalKind> TryFrom<StartCtx<K>> for TraversalContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartCtx<K>) -> Result<Self, Self::Error> {
        match start.get_parent_batch() {
            Ok(p) => Ok(Self {
                end_iter: EndIterator::new(
                    TraceContext {
                        trav: start.trav,
                        cache: TraceCache::new(start.index),
                    },
                    MatchContext {
                        nodes: FromIterator::from_iter(
                            p.into_compare_batch().into_iter().map(Parent),
                        ),
                    },
                ),
                last_end: EndState {
                    reason: EndReason::QueryEnd,
                    kind: EndKind::Complete(start.index),
                    cursor: start.cursor,
                },
            }),
            Err(end) => Err(end),
        }
    }
}

impl<K: TraversalKind> Iterator for TraversalContext<K> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        match self.end_iter.find_next() {
            Some(end) => {
                TraceStart(&end, self.last_end.start_len())
                    .trace(&mut self.end_iter.0);
                self.last_end = end;
                Some(())
            },
            None => None,
        }
    }
}

impl<'a, K: TraversalKind> HasGraph for &'a TraversalContext<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.end_iter.0.trav.graph()
    }
}

impl<'a, K: TraversalKind> HasGraph for &'a mut TraversalContext<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.end_iter.0.trav.graph()
    }
}
