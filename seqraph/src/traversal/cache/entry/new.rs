use crate::{
    traversal::{
        cache::{
            key::{
                prev::PrevKey,
                root::RootKey,
                target::TargetKey,
                DirectedKey,
                UpKey,
            },
            state::{
                child::ChildState,
                end::{
                    EndKind,
                    EndState,
                    RangeEnd,
                },
                parent::ParentState,
                InnerKind,
                StateDirection,
                TraversalState,
            },
        },
        path::accessors::{
            child::root::GraphRootChild,
            role::{
                End,
                Start,
            },
        },
        result_kind::RoleChildPath,
    },
    vertex::location::child::ChildLocation,
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
        self.prev.clone()
    }
    pub fn state_direction(&self) -> StateDirection {
        match &self.kind {
            NewKind::Parent(_) => StateDirection::BottomUp,
            NewKind::Child(_) => StateDirection::TopDown,
        }
    }
}

impl From<&TraversalState> for NewEntry {
    fn from(state: &TraversalState) -> Self {
        Self {
            prev: state.prev_key(),
            //root_pos: state.root_pos(),
            kind: (&state.kind).into(),
        }
    }
}

impl From<&InnerKind> for NewKind {
    fn from(state: &InnerKind) -> Self {
        match state {
            InnerKind::Parent(state) => Self::Parent(state.into()),
            InnerKind::Child(state) => Self::Child(state.into()),
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

impl From<&ParentState> for NewParent {
    fn from(state: &ParentState) -> Self {
        Self {
            root: state.root_key(),
            entry: state.path.role_root_child_location(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewChild {
    pub root: UpKey,
    pub target: DirectedKey,
    pub end_leaf: Option<ChildLocation>,
}

impl From<&ChildState> for NewChild {
    fn from(state: &ChildState) -> Self {
        Self {
            root: state.root_key(),
            target: state.target_key(),
            end_leaf: state.paths.path.role_leaf_child_location::<End>(),
        }
    }
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

impl From<&EndState> for NewEnd {
    fn from(state: &EndState) -> Self {
        match &state.kind {
            EndKind::Range(range) => Self::Range(range.into()),
            EndKind::Postfix(_) => Self::Postfix(state.root_key()),
            EndKind::Prefix(_) => Self::Prefix(state.target_key()),
            EndKind::Complete(_) => Self::Complete(state.target_key()),
        }
    }
}

impl From<&RangeEnd> for NewRangeEnd {
    fn from(state: &RangeEnd) -> Self {
        Self {
            target: state.target,
            entry: GraphRootChild::<Start>::root_child_location(&state.path),
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
