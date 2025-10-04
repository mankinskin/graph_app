use crate::{
    fold::foldable::ErrorState,
    r#match::{
        iterator::MatchIterator,
        MatchCtx,
        TraceNode::Parent,
    },
};
use container::StateContainer;
use context_trace::*;
use derive_new::new;
use policy::DirectedTraversalPolicy;
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
use tracing::debug;
pub mod container;
pub mod policy;
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

pub trait HasTraversalCtx<K: TraversalKind> {
    fn traversal_context(&self) -> Result<&TraversalCtx<K>, ErrorState>;
}
pub trait IntoTraversalCtx<K: TraversalKind> {
    fn into_traversal_context(self) -> Result<TraversalCtx<K>, ErrorState>;
}

/// context for generating next states
#[derive(Debug, new)]
pub struct TraversalCtx<K: TraversalKind> {
    pub match_iter: MatchIterator<K>,
    pub last_match: EndState,
}
impl<K: TraversalKind> Unpin for TraversalCtx<K> {}

impl<K: TraversalKind> IntoTraversalCtx<K> for TraversalCtx<K> {
    fn into_traversal_context(self) -> Result<TraversalCtx<K>, ErrorState> {
        Ok(self)
    }
}
impl<K: TraversalKind> IntoTraversalCtx<K> for StartCtx<K> {
    fn into_traversal_context(self) -> Result<TraversalCtx<K>, ErrorState> {
        match self.get_parent_batch() {
            Ok(p) => {
                debug!("First ParentBatch {:?}", p);
                Ok(TraversalCtx {
                    match_iter: MatchIterator::new(
                        TraceCtx {
                            trav: self.trav,
                            cache: TraceCache::new(self.index),
                        },
                        MatchCtx {
                            nodes: FromIterator::from_iter(
                                p.into_compare_batch().into_iter().map(Parent),
                            ),
                        },
                    ),
                    last_match: EndState {
                        reason: EndReason::QueryEnd,
                        kind: EndKind::Complete(self.index),
                        cursor: self.cursor,
                    },
                })
            },
            Err(end) => Err(end),
        }
    }
}
impl<K: TraversalKind> Iterator for TraversalCtx<K> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        match self.match_iter.find_next() {
            Some(end) => {
                debug!("Found end {:#?}", end);
                TraceStart(&end, self.last_match.start_len())
                    .trace(&mut self.match_iter.0);
                self.last_match = end;
                Some(())
            },
            None => None,
        }
    }
}

impl<K: TraversalKind> HasGraph for &'_ TraversalCtx<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.match_iter.0.trav.graph()
    }
}

impl<K: TraversalKind> HasGraph for &mut TraversalCtx<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.match_iter.0.trav.graph()
    }
}
