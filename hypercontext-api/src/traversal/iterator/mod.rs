use crate::{
    graph::{
        getters::NoMatch,
        vertex::wide::Wide,
    },
    path::{
        accessors::role::End,
        structs::query_range_path::QueryRangePath,
    },
    traversal::{
        cache::{
            key::root::RootKey,
            state::{
                end::{
                    EndKind,
                    EndReason,
                },
                traversal::TraversalState,
                NextStates,
                StateNext,
            },
            trace::{
                Trace,
                TraceContext,
            },
        },
        context::TraversalStateContext,
        folder::{
            state::FoldResult,
            FoldFinished,
            TraversalFolder,
        },
        iterator::traverser::{
            extend::ExtendStates,
            pruning::PruneStates,
            NodeVisitor,
            OrderedTraverser,
        },
        policy::DirectedTraversalPolicy,
        result::TraversalResult,
        traversable::TravKind,
    },
};
use std::{fmt::Debug, ops::ControlFlow};

use super::traversable::Traversable;

pub mod bands;
pub mod traverser;

pub type IterTrav<'a, It> = <It as TraversalIterator<'a>>::Trav;
pub type IterKind<'a, It> = TravKind<IterTrav<'a, It>>;
struct FoldContext {}

// Traversal Iterator Spec
//
// Traversable, Pattern -> RangePath -> TraversalIterator -> TraversalResult
//                          search
//                                       TraversalCache
//                                       TraversalState
//                                       NextStates
//                                       TraversalContext
//                                       TraversalStateContext
//                                                             FoldFound
//
//

pub struct TraversalContext<'a, Trav: Traversable> {
    trav: &'a Trav,
}
impl<'a, Trav: Traversable> TraversalContext<'a, Trav> {
}

pub trait TraversalIterator<'a>:
    Iterator<Item = (usize, TraversalState)> + Sized + ExtendStates + PruneStates + Debug
{
    type Trav: TraversalFolder + 'a;
    type Policy: DirectedTraversalPolicy<Trav = Self::Trav>;
    type NodeVisitor: NodeVisitor;

    fn trav(&self) -> &'a Self::Trav;

    //#[instrument(skip(self))]
    fn on_next_states(self, next_states: NextStates) -> ControlFlow<()>{
        match next_states {
            NextStates::Child(_) | NextStates::Prefixes(_) | NextStates::Parents(_) => {
                self.extend(
                    next_states
                        .into_states()
                        .into_iter()
                        .map(|nstate| (depth + 1, nstate)),
                );
                ControlFlow::Continue(())
            }
            NextStates::Empty => ControlFlow::Continue(()),
            NextStates::End(StateNext { inner: end, .. }) => {
                //debug!("{:#?}", state);
                if end.width() >= max_width {
                    end.trace(&mut TraceContext {
                        cache: &mut cache,
                        trav: self.trav(),
                    });

                    // note: not really needed with completion
                    //if let Some(root_key) = end.waiting_root_key() {
                    //    // continue paths also arrived at this root
                    //    // this must happen before simplification
                    //    states.extend(
                    //        cache.continue_waiting(&root_key)
                    //    );
                    //}
                    if end.width() > max_width {
                        max_width = end.width();
                        //end_states.clear();
                    }
                    let is_final = end.reason == EndReason::QueryEnd
                        && matches!(end.kind, EndKind::Complete(_));
                    end_state = Some(end);
                    is_final.then(|| ControlFlow::Break(()))
                        .unwrap_or(ControlFlow::Continue(()))
                } else {
                    // larger root already found
                    // stop other paths with this root
                    self.prune_below(end.root_key());
                    ControlFlow::Continue(())
                }
            }
        }
    }
    fn fold_states(self) -> Result<TraversalResult, (NoMatch, QueryRangePath)> {

        let mut end_state = None;
        let mut max_width = start_index.width();

        // 1. expand first parents
        // 2. expand next children/parents

        while let Some((depth, tstate)) = self.next() {
            if let Some(next_states) = {
                let mut ctx = TraversalStateContext::new(&query_root, &mut cache, &mut self);
                tstate.next_states(&mut ctx)
            } {
                if self.on_next_states(next_states).is_break() {
                    break;
                }
            }
        }
        //debug!("end roots: {:#?}", end_states.iter()
        //    .map(|s| {
        //        let root = s.root_parent();
        //        (root.index(), root.width(), s.root_pos.0)
        //    }).collect_vec()
        //);
        end_state
            .map(|state| {
                FoldFinished {
                    end_state: state,
                    cache,
                    start_index,
                    query_root,
                }
                .to_traversal_result()
            })
            .ok_or_else(|| {
                (
                    NoMatch::NotFound,
                    TraversalResult {
                        //query: query.to_rooted(query_root.query_root),
                        query: query_range_path,
                        result: FoldResult::Complete(start_index),
                    },
                )
            })
    }
}

impl<'a, Trav, S, O> TraversalIterator<'a> for OrderedTraverser<'a, Trav, S, O>
where
    Trav: TraversalFolder + 'a,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
    type Trav = Trav;
    type Policy = S;
    type NodeVisitor = O;
    fn trav(&self) -> &'a Self::Trav {
        self.trav
    }
}

impl<'a, 'b: 'a, I: TraversalIterator<'b>> TraversalIterator<'b>
    for TraversalStateContext<'a, 'b, I>
{
    type Trav = I::Trav;
    type Policy = I::Policy;
    type NodeVisitor = I::NodeVisitor;
    fn trav(&self) -> &'b Self::Trav {
        self.iter.trav()
    }
}
