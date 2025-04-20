use compare::RootCursor;
use container::StateContainer;
use context_trace::trace::{
    cache::TraceCache,
    has_graph::{
        HasGraph,
        TravKind,
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
    type Trav: HasGraph;
    type Container: StateContainer;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
}

/// context for generating next states
#[derive(Debug, new)]
pub struct TraversalContext<K: TraversalKind> {
    pub matches: MatchContext,
    #[new(default)]
    pub cache: TraceCache,
    pub trav: K::Trav,
}

impl<K: TraversalKind> TryFrom<StartCtx<K>> for TraversalContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartCtx<K>) -> Result<Self, Self::Error> {
        match start.get_parent_batch() {
            Ok(p) => Ok(Self {
                matches: MatchContext {
                    batches: FromIterator::from_iter([p]),
                    cursor: start.cursor,
                },
                trav: start.trav,
                cache: TraceCache::new(start.index),
            }),
            Err(end) => Err(end),
        }
    }
}

impl<K: TraversalKind> Iterator for TraversalContext<K> {
    type Item = OptGen<Result<ParentBatch, EndState>>;

    fn next(&mut self) -> Option<Self::Item> {
        match MatchIterator::<K>::new(&self.trav, &mut self.matches).next() {
            Some(Yield(root_cursor)) => Some(
                // TODO: add cache for path to parent
                match root_cursor.find_end() {
                    // TODO: add cache for end
                    Err(root_cursor) => match root_cursor
                        .state
                        .root_parent()
                        .next_parents::<K>(&self.trav)
                    {
                        Ok(batch) => {
                            // next batch
                            self.matches.batches.push_back(batch);
                            Pass
                        },
                        // TODO: if no new batch, return end state
                        Err(end) => Yield(Err(end)),
                    },
                    Ok(end) => Yield(Err(end)),
                },
            ),
            Some(Pass) => Some(Pass),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OptGen<Y> {
    Yield(Y),
    Pass,
}
use OptGen::*;
/// context for generating next states
#[derive(Debug, new)]
pub struct MatchContext {
    #[new(default)]
    pub batches: VecDeque<ParentBatch>,
    pub cursor: PatternRangeCursor,
}
#[derive(Debug, new)]
pub struct MatchIterator<'a, K: TraversalKind>(
    &'a K::Trav,
    &'a mut MatchContext,
);

impl<'a, K: TraversalKind> Iterator for MatchIterator<'a, K> {
    type Item = OptGen<RootCursor<&'a K::Trav>>;

    fn next(&mut self) -> Option<Self::Item> {
        let MatchIterator(trav, ctx) = self;
        if let Some(batch) = ctx.batches.pop_front() {
            // one parent level (batched)
            Some(
                match ParentBatchChildren::<&'a K::Trav>::new(*trav, batch)
                    // find parent with a match
                    .find_root_cursor()
                {
                    // root found
                    Break(root_cursor) => {
                        // drop other candidates
                        ctx.batches.clear();
                        // TODO: add cache for path to parent
                        Yield(root_cursor)
                    },
                    // continue with
                    Continue(next) => {
                        ctx.batches.extend(next.into_iter().flat_map(
                            |parent| K::Policy::next_batch(&trav, &parent),
                        ));
                        Pass
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

impl<'a, K: TraversalKind> HasGraph for &'a TraversalContext<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}

impl<'a, K: TraversalKind> HasGraph for &'a mut TraversalContext<K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g>
        = <K::Trav as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
