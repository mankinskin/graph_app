use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartState {
    pub index: Child,
    pub query: QueryState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentState {
    pub prev_pos: TokenLocation,
    pub root_pos: TokenLocation,
    pub path: Primer,
    pub query: QueryState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildState {
    pub prev_pos: TokenLocation,
    pub root_pos: TokenLocation,
    pub target: DirectedKey,
    pub paths: PathPair,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState {
    pub prev: DirectedKey,
    pub new: Vec<NewEntry>,
    pub kind: InnerKind,
}
impl Ord for TraversalState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind.cmp(&other.kind)
    }
}
impl PartialOrd for TraversalState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for InnerKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            //(InnerKind::End(_), InnerKind::End(_)) => Ordering::Equal,
            (InnerKind::Child(a), InnerKind::Child(b)) => a.cmp(b),
            (InnerKind::Parent(a), InnerKind::Parent(b)) => a.cmp(b),
            (InnerKind::Child(_), _) => Ordering::Less,
            //(InnerKind::End(_), _)
            (_, InnerKind::Child(_)) => Ordering::Greater,
            //(_, InnerKind::End(_))
        }
    }
}
impl PartialOrd for InnerKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ParentState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.root_parent().cmp(
            &other.path.root_parent()
        )
    }
}
impl Ord for Child {
    fn cmp(&self, other: &Self) -> Ordering {
        self.width().cmp(&other.width())
    }
}
impl PartialOrd for ParentState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ChildState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.paths.path.root_parent().cmp(
            &other.paths.path.root_parent()
        )
    }
}
impl PartialOrd for ChildState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl TraversalState {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            InnerKind::Parent(state) => Some(state.path.root_child_location()),
            InnerKind::Child(state) => state.paths.path.role_leaf_child_location::<End>(),
            //InnerKind::End(state) => state.entry_location(),
        }
    }
    pub fn prev_key(&self) -> DirectedKey {
        self.prev
    }
    pub fn root_pos(&self) -> TokenLocation {
        match &self.kind {
            InnerKind::Parent(state)
                => state.root_pos,
            InnerKind::Child(state)
                => state.root_pos,
            //InnerKind::End(state)
            //    => state.root_pos
        }
    }
    //pub fn prev_pos(&self) -> TokenLocation {
    //    match &self.kind {
    //        InnerKind::Parent(state)
    //            => state.prev_pos,
    //        InnerKind::Child(state)
    //            => state.prev_pos,
    //        InnerKind::End(state)
    //            => state.prev_pos
    //    }
    //}
    pub fn node_direction(&self) -> NodeDirection {
        match &self.kind {
            InnerKind::Parent(_)
                => NodeDirection::BottomUp,
            InnerKind::Child(_)
                => NodeDirection::TopDown,
            //InnerKind::End(state) => state.node_direction()
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InnerKind {
    Parent(ParentState),
    Child(ChildState),
    //End(EndState),
}
impl From<WaitingState> for TraversalState {
    fn from(state: WaitingState) -> Self {
        Self {
            prev: state.prev,
            //matched: state.matched,
            new: vec![],
            kind: InnerKind::Parent(state.state),
            //query: state.query,
        }
    }
}