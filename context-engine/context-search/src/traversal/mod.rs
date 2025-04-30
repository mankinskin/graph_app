use compare::RootCursor;
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
use iterator::policy::DirectedTraversalPolicy;
use itertools::Itertools;
use state::{
    end::{
        EndKind,
        EndReason,
        EndState,
    },
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
    pub last_end: EndState,
}

impl<K: TraversalKind> TryFrom<StartCtx<K>> for TraversalContext<K> {
    type Error = ErrorState;
    fn try_from(start: StartCtx<K>) -> Result<Self, Self::Error> {
        match start.get_parent_batch() {
            Ok(p) => Ok(Self {
                matches: MatchContext {
                    batches: FromIterator::from_iter([p]),
                },
                trav: start.trav,
                cache: TraceCache::new(start.index),
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
        match EndIterator::<K>::new(&self.trav, &mut self.matches).next() {
            Some(Yield(end)) => {
                //assert!(
                //    end.width() >= self.last_end.width(),
                //    "Parents not evaluated in order"
                //);
                // TODO: add cache for front of end
                // TODO: only add cache until last matched parent
                match &end.kind {
                    EndKind::Postfix(post) => {
                        let mut ctx = TraceContext {
                            cache: &mut self.cache,
                            trav: &self.trav,
                        };
                        post.clone().trace(&mut ctx);
                    },
                    _ => {},
                };

                self.last_end = end;
                (!self.last_end.is_final()).then_some(())
            },
            Some(Pass) => Some(()),
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

#[derive(Debug, new)]
pub struct EndIterator<'a, K: TraversalKind>(&'a K::Trav, &'a mut MatchContext);

impl<'a, K: TraversalKind> Iterator for EndIterator<'a, K> {
    type Item = OptGen<EndState>;

    fn next(&mut self) -> Option<Self::Item> {
        match MatchIterator::<K>::new(self.0, self.1).next() {
            Some(Yield(root_cursor)) => Some(Yield(
                // add cache for path to parent
                // TODO: add cache for end
                match root_cursor.find_end() {
                    Ok(end) => end,
                    Err(root_cursor) => match root_cursor
                        .state
                        .root_parent()
                        .next_parents::<K>(&self.0)
                    {
                        // TODO: if no new batch, return end state
                        Err(end) => end,
                        Ok((parent, batch)) => {
                            assert!(!batch.is_empty());
                            // next batch
                            self.1.batches.push_back(batch);
                            EndState {
                                reason: EndReason::Mismatch,
                                kind: EndKind::from_start_path(
                                    parent.path,
                                    parent.root_pos,
                                    self.0,
                                ),
                                cursor: parent.cursor,
                            }
                        },
                    },
                },
            )),
            Some(Pass) => Some(Pass),
            None => None,
        }
    }
}

/// context for generating next states
#[derive(Debug, new)]
pub struct MatchContext {
    #[new(default)]
    pub batches: VecDeque<ParentBatch>,
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
                        ctx.batches.extend(next.into_iter().sorted().flat_map(
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
