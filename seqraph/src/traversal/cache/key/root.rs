use crate::*;

pub trait RootKey {
    fn root_key(&self) -> UpKey;
}
impl RootKey for ParentState {
    fn root_key(&self) -> UpKey {
        UpKey::new(
            self.path.root_parent(),
            self.root_pos.into(),
        )
    }
}
impl RootKey for StartState {
    fn root_key(&self) -> UpKey {
        UpKey::new(
            self.index,
            TokenLocation(self.index.width()).into(),
        )
    }
}
impl RootKey for ChildState {
    fn root_key(&self) -> UpKey {
        UpKey::new(
            self.paths.path.root_parent(),
            self.root_pos.into(),
        )
    }
}
impl RootKey for TraversalState {
    fn root_key(&self) -> UpKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.root_key(),
        }
    }
}
impl RootKey for EndState {
    fn root_key(&self) -> UpKey {
        UpKey::new(
            match &self.kind {
                EndKind::Range(s) => s.path.root_parent(),
                EndKind::Postfix(p) => p.path.root_parent(),
                EndKind::Prefix(p) => p.path.root_parent(),
                EndKind::Complete(c) => *c,
            },
            self.root_pos.into(),
        )
    }
}
