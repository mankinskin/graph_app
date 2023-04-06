use crate::*;

pub trait TargetKey {
    fn target_key(&self) -> DirectedKey;
}
impl TargetKey for TraversalState {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.leaf_key(),
            //InnerKind::End(state) => state.target_key(),
        }
    }
}
impl TargetKey for ChildState {
    fn target_key(&self) -> DirectedKey {
        self.target
    }
}
impl TargetKey for EndState {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            EndKind::Range(p) => p.target,//DirectedKey::new(state.path.role_leaf_child::<End, _>(trav), *self.query_pos()),
            EndKind::Postfix(_) => self.root_key(),
            EndKind::Prefix(p) => p.target,
            EndKind::Complete(c) => DirectedKey::up(*c, *self.query_pos()),
        }
    }
}
impl TargetKey for NewEntry {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            NewKind::Parent(state) => state.root,
            NewKind::Child(state) => state.target,
            //NewKind::End(state) => state.target_key(),
        }
    }
}
impl TargetKey for NewEnd {
    fn target_key(&self) -> DirectedKey {
        match &self {
            Self::Range(state) => state.target,//DirectedKey::new(state.path.role_leaf_child::<End, _>(trav), *self.query_pos()),
            Self::Postfix(root) => *root,
            Self::Prefix(target) |
            Self::Complete(target) => *target,
        }
    }
}