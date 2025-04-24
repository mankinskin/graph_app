use crate::{
    graph::vertex::location::child::ChildLocation,
    trace::{
        StateDirection,
        cache::key::{
            directed::{
                DirectedKey,
                up::UpKey,
            },
            props::TargetKey,
        },
    },
};

//impl NewEntry {
//pub fn entry_location(&self) -> Option<ChildLocation> {
//    match &self.kind {
//        EditKind::Parent(state) => Some(state.entry.clone()),
//        EditKind::Child(state) => state.end_leaf.clone(),
//    }
//}
//pub fn prev_key(&self) -> PrevKey {
//    self.prev.clone()
//}
//pub fn state_direction(&self) -> StateDirection {
//    match &self.kind {
//        EditKind::Parent(_) => StateDirection::BottomUp,
//        EditKind::Child(_) => StateDirection::TopDown,
//    }
//}
//}
use derive_more::From;

use super::key::directed::down::DownKey;

#[derive(Clone, Debug, PartialEq, Eq, From)]
pub enum EditKind {
    Parent(UpEdit),
    Child(DownEdit),
    //Root(RootEdit),
}

impl TargetKey for EditKind {
    fn target_key(&self) -> DirectedKey {
        match &self {
            EditKind::Parent(state) => state.target.clone().into(),
            //EditKind::Root(state) => state.entry_key.clone().into(),
            EditKind::Child(state) => state.target.clone().into(),
        }
    }
}
pub trait Edit: Into<EditKind> {}
impl<T: Into<EditKind>> Edit for T {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpEdit {
    pub target: UpKey,
    pub location: ChildLocation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootEdit {
    pub entry_key: UpKey,
    pub entry_location: ChildLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownEdit {
    //pub root: UpKey,
    pub prev: DownKey,
    pub target: DownKey,
    pub location: ChildLocation,
    //pub end_leaf: Option<ChildLocation>,
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
    pub fn state_direction(&self) -> StateDirection {
        match self {
            Self::Range(_) => StateDirection::TopDown,
            Self::Postfix(_) => StateDirection::BottomUp,
            Self::Prefix(_) => StateDirection::TopDown,
            Self::Complete(_) => StateDirection::BottomUp,
        }
    }
}
