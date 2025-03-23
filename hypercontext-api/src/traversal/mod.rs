use cache::{
    key::{
        directed::DirectedKey,
        props::TargetKey,
    },
    TraversalCache,
};
use container::StateContainer;
use iterator::policy::DirectedTraversalPolicy;
use state::{
    bottom_up::{
        parent::{
            ParentNext,
            ParentState,
        },
        BUNext,
    },
    top_down::{
        child::{
            ChildState,
            MatchedNext,
            TDNext,
        },
        end::{
            EndReason,
            EndState,
            RangeEnd,
        },
    },
    BaseState,
    StateNext,
};
use std::{
    collections::VecDeque,
    fmt::Debug,
};
use traversable::Traversable;

use crate::{
    graph::vertex::wide::Wide,
    path::{
        accessors::role::End,
        mutators::move_path::{
            key::AdvanceKey,
            Advance,
        },
        RoleChildPath,
    },
    traversal::cache::key::prev::ToPrev,
};
pub mod cache;
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

//  1. Input
//      - Pattern
//      - QueryState
//  2. Init
//      - Trav
//      - start index
//      - start states
//  3. Fold
//      - TraversalCache
//      - FoldStepState

/// context for generating next states
#[derive(Debug)]
pub struct TraversalContext<K: TraversalKind> {
    //pub states: PrunedStates<K>,
    pub parents: VecDeque<ParentState>,
    pub children: VecDeque<ChildState>,
    pub end: Vec<EndState>,
    pub cache: TraversalCache,
    pub trav: K::Trav,
}

impl<K: TraversalKind> Iterator for TraversalContext<K> {
    type Item = (usize, Option<EndState>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cs) = self.children.pop_front() {
            let end = match cs.child_next_states(self) {
                TDNext::Mismatched(end) => Some(end.inner),
                TDNext::Prefixes(next) => {
                    self.children.extend(next.inner);
                    None
                }
                TDNext::Matched(next) => {
                    let primed = match next {
                        MatchedNext::NextChild(next_child) => self
                            .try_advance_query(next_child)
                            .map(MatchedNext::NextChild),
                        MatchedNext::MatchedParent(cs) => {
                            self.try_advance_query(cs).map(MatchedNext::MatchedParent)
                        }
                    };
                    match primed {
                        Ok(MatchedNext::NextChild(next_child)) => {
                            self.children.extend([next_child]);
                            None
                        }
                        Ok(MatchedNext::MatchedParent(next_child)) => {
                            //ParentState {
                            //    path: IndexStartPath::from(self.base.path),
                            //    ..self.base
                            //}
                            //.next_parents::<K>(&ctx.trav)

                            //self.parents.extend(
                            //    next_child
                            //        .root_parent
                            //);
                            None
                        }
                        Err(end) => Some(end),
                    }
                }
            };

            Some((0, end))
        } else if let Some(ps) = self.parents.pop_front() {
            if self.cache.exists(&ps.target_key()) {
                Some((0, None))
            } else {
                let end = match ps.parent_next_states::<K>(&self.trav) {
                    ParentNext::Child(next_child) => {
                        self.children.extend([next_child.inner]);
                        None
                    }
                    ParentNext::BU(bu_next) => match bu_next {
                        BUNext::Parents(p) => {
                            self.parents.extend(p.inner);
                            None
                        }
                        BUNext::End(end) => Some(end.inner),
                    },
                };
                Some((0, end))
            }
        } else {
            None
        }

        //self.states.extend(
        //    next_states
        //        .into_states()
        //        .into_iter()
        //        .map(|nstate| (depth + 1, nstate)),
        //);
    }
}

impl<K: TraversalKind> Unpin for TraversalContext<K> {}

impl<K: TraversalKind> TraversalContext<K> {
    pub fn add_root_candidate(&mut self) {
        self.children.clear();
        //ctx.cache.add_state(
        //    &ctx.trav,
        //    TraversalState::from((self.root_prev, self.root_parent.clone())),
        //    true,
        //);
    }
    pub fn try_advance_query(
        &self,
        mut state: ChildState,
    ) -> Result<ChildState, EndState> {
        if state.cursor.advance(&self.trav).is_continue() {
            Ok(state)
        } else {
            // query ended
            let key = state.target_key();
            let BaseState {
                mut cursor,
                path,
                root_pos,
                ..
            } = state.base;
            let target_index = path.role_leaf_child::<End, _>(&self.trav);
            let pos = cursor.relative_pos;
            cursor.advance_key(target_index.width());
            Err(EndState {
                root_pos,
                cursor,
                reason: EndReason::QueryEnd,
                kind: RangeEnd {
                    path,
                    target: DirectedKey::down(target_index, pos),
                }
                .simplify_to_end(&self.trav),
            })
        }
    }
}
