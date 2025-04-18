use container::StateContainer;
use context_trace::trace::{
    cache::TraceCache,
    traversable::{
        TravKind,
        Traversable,
    },
};
use derive_new::new;
use fold::foldable::ErrorState;
use iterator::policy::DirectedTraversalPolicy;
use state::{
    cursor::PatternRangeCursor,
    end::EndState,
    parent::batch::{
        ParentBatch,
        ParentBatchChildren,
    },
    start::StartCtx,
    BaseState,
};
use std::{
    collections::VecDeque,
    fmt::Debug,
    ops::ControlFlow::{
        self,
        Break,
        Continue,
    },
};

pub mod compare;
pub mod container;
pub mod fold;
pub mod iterator;
pub mod result;
pub mod state;

pub trait TraversalKind: Debug + Default {
    type Trav: Traversable;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}

/// context for generating next states
#[derive(Debug, new)]
pub struct TraversalContext<K: TraversalKind> {
    pub trav: K::Trav,
    #[new(default)]
    pub batches: VecDeque<ParentBatch>,
    #[new(default)]
    pub cache: TraceCache,
    pub cursor: PatternRangeCursor,
}

impl<K: TraversalKind> TryFrom<StartCtx<K>> for TraversalContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartCtx<K>) -> Result<Self, Self::Error> {
        match start.get_parent_batch() {
            Ok(p) => Ok(Self {
                batches: FromIterator::from_iter([p]),
                cache: TraceCache::new(&start.trav, start.index),
                trav: start.trav,
                cursor: start.cursor,
            }),
            Err(end) => Err(end),
        }
    }
}

impl<K: TraversalKind> Iterator for TraversalContext<K> {
    type Item = ControlFlow<EndState>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(batch) = self.batches.pop_front() {
            // one parent level (batched)
            Some(
                match ParentBatchChildren::new(&self.trav, batch)
                    // find parent with a match
                    .find_root_cursor()
                {
                    // root found
                    Break((root_parent, root_cursor)) => {
                        // drop other candidates
                        self.batches.clear();
                        // TODO: add cache for path to parent
                        if let Some(end) = root_cursor.find_end() {
                            // TODO: add cache for end
                            Break(end)
                        } else {
                            // TODO: if no new batch, return end state
                            if let Some(next) =
                                K::Policy::next_batch(&self.trav, &root_parent)
                            {
                                self.batches.push_back(next);
                            }
                            // next batch
                            Continue(())
                        }
                    },
                    // continue with
                    Continue(next) => {
                        self.batches.extend(next.into_iter().flat_map(
                            |parent| K::Policy::next_batch(&self.trav, &parent),
                        ));
                        Continue(())
                    },
                },
            )
        } else {
            // no more parents
            None
        }
    }
}

impl<K: TraversalKind> Unpin for TraversalContext<K> {}

impl<'a, K: TraversalKind> Traversable for &'a TraversalContext<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as Traversable>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}

impl<'a, K: TraversalKind> Traversable for &'a mut TraversalContext<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as Traversable>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
