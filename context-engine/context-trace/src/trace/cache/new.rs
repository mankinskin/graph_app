use crate::{
    graph::vertex::location::child::ChildLocation,
    trace::{
        BottomUp,
        TopDown,
        TraceDirection,
        cache::key::{
            directed::{
                DirectedKey,
                up::UpKey,
            },
            props::TargetKey,
        },
    },
};

use derive_more::From;
use derive_new::new;

#[derive(Clone, Debug, PartialEq, Eq, From)]
pub enum EditKind {
    Parent(NewTraceEdge<BottomUp>),
    Child(NewTraceEdge<TopDown>),
}

impl TargetKey for EditKind {
    fn target_key(&self) -> DirectedKey {
        match &self {
            EditKind::Parent(state) => state.target.clone().into(),
            EditKind::Child(state) => state.target.clone().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct NewTraceEdge<D: TraceDirection> {
    pub prev: D::Key,
    pub target: D::Key,
    pub location: ChildLocation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootEdit {
    pub entry_key: UpKey,
    pub entry_location: ChildLocation,
}
