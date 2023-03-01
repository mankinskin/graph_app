use crate::*;

pub trait RootKey {
    fn root_key(&self) -> CacheKey;
}
//impl RootKey for IndexRoot {
//    fn root_key(&self) -> CacheKey {
//        CacheKey::new(self.location.parent, self.pos)
//    }
//}
//impl RootKey for SearchPath {
//    fn root_key(&self) -> CacheKey {
//        self.root.root_key()
//    }
//}
//impl RootKey for PathPair {
//    fn root_key(&self) -> CacheKey {
//        self.path.root_key()
//    }
//}
impl RootKey for ParentState {
    fn root_key(&self) -> CacheKey {
        CacheKey::new(
            self.path.root_parent(),
            self.root_pos,
        )
    }
}
//impl RootKey for RootedRolePath<Start, IndexRoot> {
//    fn root_key(&self) -> CacheKey {
//        self.split_path.root.root_key()
//    }
//}
impl RootKey for StartState {
    fn root_key(&self) -> CacheKey {
        CacheKey::new(self.index, *self.query_pos())
    }
}
impl RootKey for ChildState {
    fn root_key(&self) -> CacheKey {
        CacheKey::new(
            self.paths.path.root_parent(),
            self.root_pos,
        )
    }
}
impl RootKey for TraversalState {
    fn root_key(&self) -> CacheKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.root_key(),
            InnerKind::End(state) => state.root_key(),
        }
    }
}
impl RootKey for EndState {
    fn root_key(&self) -> CacheKey {
        match &self.kind {
            EndKind::Range(s) => CacheKey::new(s.path.root_parent(), self.root_pos),
            EndKind::Postfix(path) => CacheKey::new(path.root_parent(), self.root_pos),
            EndKind::Prefix(path) => CacheKey::new(path.path.root_parent(), self.root_pos),
            EndKind::Complete(c) => CacheKey::new(*c, self.root_pos),
        }
    }
}
