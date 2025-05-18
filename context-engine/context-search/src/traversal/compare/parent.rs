use crate::traversal::state::cursor::PatternPrefixCursor;
use context_trace::{
    path::mutators::adapters::IntoAdvanced,
    trace::{
        has_graph::HasGraph,
        state::parent::ParentState,
    },
};
use derive_more::{
    Deref,
    DerefMut,
};
use std::fmt::Debug;

use super::state::CompareState;
use crate::traversal::compare::state::PathPairMode::GraphMajor;
use context_trace::{
    graph::vertex::child::Child,
    trace::cache::key::directed::down::DownKey,
};
#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct CompareRootState {
    #[deref]
    #[deref_mut]
    pub child: CompareState,
    pub root_parent: ParentState,
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ParentCompareState {
    #[deref]
    #[deref_mut]
    pub parent_state: ParentState,
    pub cursor: PatternPrefixCursor,
}
impl IntoAdvanced for ParentCompareState {
    type Next = CompareRootState;
    fn into_advanced<G: HasGraph>(
        self,
        trav: &G,
    ) -> Result<Self::Next, Self> {
        match self.parent_state.into_advanced(trav) {
            Ok(next) => Ok(CompareRootState {
                child: CompareState {
                    child_state: next.child,
                    cursor: self.cursor,
                    mode: GraphMajor,
                    target: DownKey::new(Child::new(0, 0), 0.into()),
                },
                root_parent: next.root_parent,
            }),
            Err(parent_state) => Err(Self {
                parent_state,
                cursor: self.cursor,
            }),
        }
    }
}
