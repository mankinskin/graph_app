use crate::{
    graph::vertex::location::child::ChildLocation,
    trace::{
        StateDirection,
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
            NewKind::Parent(state) => Some(state.entry),
            NewKind::Child(state) => state.end_leaf,
        }
    }
    pub fn prev_key(&self) -> PrevKey {
        self.prev
    }
    pub fn state_direction(&self) -> StateDirection {
        match &self.kind {
            NewKind::Parent(_) => StateDirection::BottomUp,
            NewKind::Child(_) => StateDirection::TopDown,
        }
    }
}
impl TargetKey for NewEntry {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            NewKind::Parent(state) => state.root.into(),
            NewKind::Child(state) => state.target,
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
            Self::Range(state) => state.target,
            Self::Postfix(root) => (*root).into(),
            Self::Prefix(target) | Self::Complete(target) => *target,
        }
    }
}

impl NewEnd {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match self {
            Self::Range(state) => Some(state.entry),
            Self::Postfix(_) => None,
            Self::Prefix(_) => None,
            Self::Complete(_) => None,
        }
    }
    pub fn state_direction(&self) -> StateDirection {
        match self {
            Self::Range(_) => StateDirection::TopDown,
            Self::Postfix(_) => StateDirection::BottomUp,
            Self::Prefix(_) => StateDirection::TopDown,
            Self::Complete(_) => StateDirection::BottomUp,
        }
    }
}
