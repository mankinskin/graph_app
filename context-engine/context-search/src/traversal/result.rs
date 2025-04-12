use crate::{
    graph::vertex::child::Child,
    path::accessors::complete::PathComplete,
};

use super::{
    cache::TraversalCache,
    state::top_down::end::{
        EndKind,
        EndState,
    },
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FinishedKind {
    Complete(Child),
    Incomplete(EndState),
}

impl From<EndState> for FinishedKind {
    fn from(state: EndState) -> Self {
        if let EndKind::Complete(c) = &state.kind {
            FinishedKind::Complete(*c) // cursor.path
        } else {
            FinishedKind::Incomplete(state)
        }
    }
}
impl FinishedKind {
    pub fn unwrap_complete(self) -> Child {
        self.expect_complete("Unable to unwrap complete FoundRange")
    }
    pub fn unwrap_incomplete(self) -> EndState {
        self.expect_incomplete("Unable to unwrap incomplete FoundRange")
    }
    pub fn expect_complete(
        self,
        msg: &str,
    ) -> Child {
        match self {
            Self::Complete(c) => c,
            _ => panic!("{}", msg),
        }
    }
    pub fn expect_incomplete(
        self,
        msg: &str,
    ) -> EndState {
        match self {
            Self::Incomplete(s) => s,
            _ => panic!("{}", msg),
        }
    }
}

impl PathComplete for FinishedKind {
    /// returns child if reduced to single child
    fn as_complete(&self) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteState {
    pub cache: TraversalCache,
    pub root: Child,
    pub start: Child,
}
impl TryFrom<FinishedState> for CompleteState {
    type Error = IncompleteState;
    fn try_from(value: FinishedState) -> Result<Self, Self::Error> {
        match value {
            FinishedState {
                kind: FinishedKind::Incomplete(end_state),
                cache,
                root,
                start,
            } => Err(IncompleteState {
                end_state,
                cache,
                root,
                start,
            }),
            FinishedState {
                kind: FinishedKind::Complete(_),
                cache,
                root,
                start,
            } => Ok(CompleteState { cache, root, start }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteState {
    pub end_state: EndState,
    pub cache: TraversalCache,
    pub root: Child,
    pub start: Child,
}
impl TryFrom<FinishedState> for IncompleteState {
    type Error = CompleteState;
    fn try_from(value: FinishedState) -> Result<Self, Self::Error> {
        match CompleteState::try_from(value) {
            Ok(x) => Err(x),
            Err(x) => Ok(x),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedState {
    pub kind: FinishedKind,
    pub cache: TraversalCache,
    pub root: Child,
    pub start: Child,
}

impl FinishedState {
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.kind.unwrap_complete()
    }
    #[allow(unused)]
    #[track_caller]
    pub fn expect_complete(
        self,
        msg: &str,
    ) -> Child {
        self.kind.expect_complete(msg)
    }
}
