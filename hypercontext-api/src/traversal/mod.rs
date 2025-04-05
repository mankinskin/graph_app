use batch::{
    ParentBatch,
    ParentBatchChildren,
};
use cache::TraversalCache;
use container::StateContainer;
use fold::foldable::ErrorState;
use iterator::policy::DirectedTraversalPolicy;
use state::{
    bottom_up::start::StartContext,
    top_down::end::EndState,
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
use traversable::Traversable;

pub mod batch;
pub mod cache;
pub mod compare;
pub mod container;
pub mod fold;
pub mod iterator;
pub mod result;
pub mod split;
pub mod state;
pub mod traversable;

pub trait TraversalKind: Debug + Default {
    type Trav: Traversable;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}

/// context for generating next states
#[derive(Debug)]
pub struct TraversalContext<K: TraversalKind> {
    pub batches: VecDeque<ParentBatch>,
    pub cache: TraversalCache,
    pub trav: K::Trav,
}

impl<K: TraversalKind> TryFrom<StartContext<K>> for TraversalContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartContext<K>) -> Result<Self, Self::Error> {
        match start.state.get_parent_batch::<K>(&start.trav) {
            Ok(p) => Ok(Self {
                batches: FromIterator::from_iter([p]),
                cache: TraversalCache::new(&start.trav, start.state.index),
                trav: start.trav,
            }),
            Err(end) => Err(end),
        }
    }
}
impl<K: TraversalKind> TraversalContext<K> {
    fn new(trav: K::Trav) -> Self {
        Self {
            trav,
            cache: Default::default(),
            batches: Default::default(),
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
                        // TODO: add cache for path to parent
                        if let Some(end) = root_cursor.find_end() {
                            // TODO: add cache for end
                            Break(end)
                        } else {
                            if let Some(next) = K::Policy::next_batch(&self.trav, &root_parent) {
                                self.batches.push_back(next);
                            }
                            // next batch
                            Continue(())
                        }
                    }
                    // continue with
                    Continue(next) => {
                        self.batches.push_back(next);
                        Continue(())
                    }
                },
            )
        } else {
            // no more parents
            None
        }
    }
}

impl<K: TraversalKind> Unpin for TraversalContext<K> {}
