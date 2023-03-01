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
    pub matched: bool,
    pub path: Primer,
    pub query: QueryState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildState {
    pub prev_pos: TokenLocation,
    pub root_pos: TokenLocation,
    pub target: CacheKey,
    pub matched: bool,
    pub paths: PathPair,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState {
    pub prev: CacheKey,
    //pub matched: bool,
    pub kind: InnerKind,
}
impl TraversalState {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            InnerKind::Parent(state) => Some(state.path.root_child_location()),
            InnerKind::Child(state) => state.paths.path.role_leaf_child_location::<End>(),
            InnerKind::End(state) => state.entry_location(),
        }
    }
    pub fn prev_key(&self) -> CacheKey {
        self.prev
    }
    pub fn node_direction(&self) -> NodeDirection {
        match &self.kind {
            InnerKind::Parent(_)
                => NodeDirection::BottomUp,
            InnerKind::Child(_)
                => NodeDirection::TopDown,
            InnerKind::End(state) => state.node_direction()
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InnerKind {
    Parent(ParentState),
    Child(ChildState),
    End(EndState),
}
impl From<WaitingState> for TraversalState {
    fn from(state: WaitingState) -> Self {
        Self {
            prev: state.prev,
            //matched: state.matched,
            kind: InnerKind::Parent(state.state),
            //query: state.query,
        }
    }
}