use crate::*;

pub trait RootKey {
    fn root_key(&self) -> DirectedKey;
}
//impl RootKey for IndexRoot {
//    fn root_key(&self) -> DirectedKey {
//        DirectedKey::new(self.location.parent, self.pos)
//    }
//}
//impl RootKey for SearchPath {
//    fn root_key(&self) -> DirectedKey {
//        self.root.root_key()
//    }
//}
//impl RootKey for PathPair {
//    fn root_key(&self) -> DirectedKey {
//        self.path.root_key()
//    }
//}
impl RootKey for ParentState {
    fn root_key(&self) -> DirectedKey {
        DirectedKey::up(
            self.path.root_parent(),
            self.root_pos,
        )
    }
}
//impl RootKey for RootedRolePath<Start, IndexRoot> {
//    fn root_key(&self) -> DirectedKey {
//        self.split_path.root.root_key()
//    }
//}
impl RootKey for StartState {
    fn root_key(&self) -> DirectedKey {
        DirectedKey::up(self.index, self.index.width())
    }
}
impl RootKey for ChildState {
    fn root_key(&self) -> DirectedKey {
        DirectedKey::up(
            self.paths.path.root_parent(),
            self.root_pos,
        )
    }
}
impl RootKey for TraversalState {
    fn root_key(&self) -> DirectedKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.root_key(),
            //InnerKind::End(state) => state.root_key(),
        }
    }
}
impl RootKey for EndState {
    fn root_key(&self) -> DirectedKey {
        match &self.kind {
            EndKind::Range(s) => DirectedKey::up(s.path.root_parent(), self.root_pos),
            EndKind::Postfix(p) => DirectedKey::up(p.path.root_parent(), self.root_pos),
            EndKind::Prefix(p) => DirectedKey::up(p.path.root_parent(), self.root_pos),
            EndKind::Complete(c) => DirectedKey::up(*c, self.root_pos),
        }
    }
}
