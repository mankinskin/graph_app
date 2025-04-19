use crate::{
    graph::vertex::location::child::ChildLocation,
    trace::{
        TraceDirection,
        cache::key::{
            directed::{
                DirectedKey,
                up::UpKey,
            },
            prev::PrevKey,
            props::TargetKey,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewEntry {
    pub prev: PrevKey,
    pub kind: NewKind,
}

impl NewEntry {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            NewKind::Parent(state) => Some(state.entry.clone()),
            NewKind::Child(state) => state.end_leaf.clone(),
        }
    }
    pub fn prev_key(&self) -> PrevKey {
        self.prev.clone()
    }
    pub fn state_direction(&self) -> TraceDirection {
        match &self.kind {
            NewKind::Parent(_) => TraceDirection::BottomUp,
            NewKind::Child(_) => TraceDirection::TopDown,
        }
    }
}
impl TargetKey for NewEntry {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            NewKind::Parent(state) => state.root.clone().into(),
            NewKind::Child(state) => state.target.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewKind {
    Parent(NewParent),
    Child(NewChild),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewParent {
    pub root: UpKey,
    pub entry: ChildLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewChild {
    pub root: UpKey,
    pub target: DirectedKey,
    pub end_leaf: Option<ChildLocation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewRangeEnd {
    pub target: DirectedKey,
    pub entry: ChildLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewEnd {
    Range(NewRangeEnd),
    Postfix(UpKey),
    Prefix(DirectedKey),
    Complete(DirectedKey),
}

impl TargetKey for NewEnd {
    fn target_key(&self) -> DirectedKey {
        match &self {
            Self::Range(state) => state.target.clone(),
            Self::Postfix(root) => root.clone().into(),
            Self::Prefix(target) | Self::Complete(target) => target.clone(),
        }
    }
}

impl NewEnd {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match self {
            Self::Range(state) => Some(state.entry.clone()),
            Self::Postfix(_) => None,
            Self::Prefix(_) => None,
            Self::Complete(_) => None,
        }
    }
    pub fn state_direction(&self) -> TraceDirection {
        match self {
            Self::Range(_) => TraceDirection::TopDown,
            Self::Postfix(_) => TraceDirection::BottomUp,
            Self::Prefix(_) => TraceDirection::TopDown,
            Self::Complete(_) => TraceDirection::BottomUp,
        }
    }
}
