use crate::*;

pub trait TargetKey {
    fn target_key(&self) -> DirectedKey;
}
impl TargetKey for TraversalState {
    fn target_key(&self) -> DirectedKey {
        match &self.kind {
            InnerKind::Parent(state) => state.target_key(),
            InnerKind::Child(state) => state.target_key(),
        }
    }
}
impl TargetKey for ParentState {
    fn target_key(&self) -> DirectedKey {
        self.root_key().into()
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
            EndKind::Range(p) => p.target,
            EndKind::Postfix(_) => self.root_key().into(),
            EndKind::Prefix(p) => p.target,
            EndKind::Complete(c) => DirectedKey::up(*c, *self.query_pos()),
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
impl TargetKey for NewEnd {
    fn target_key(&self) -> DirectedKey {
        match &self {
            Self::Range(state) => state.target,
            Self::Postfix(root) => (*root).into(),
            Self::Prefix(target) |
            Self::Complete(target) => *target,
        }
    }
}