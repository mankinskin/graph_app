pub mod pos;

use crate::*;
pub use pos::*; 

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub index: Child,
    pub pos: TokenLocation,
}
impl CacheKey {
    pub fn new(index: Child, pos: impl Into<TokenLocation>) -> Self {
        Self {
            index,
            pos: pos.into(),
        }
    }
}
impl From<Child> for CacheKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: index.width().into(),
        }
    }
}

pub trait GetCacheKey: RootKey + LeafKey {
    fn leaf_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey;
}

pub trait LeafKey {
    fn leaf_location(&self) -> ChildLocation;
}
impl LeafKey for SearchPath {
    fn leaf_location(&self) -> ChildLocation {
        self.end.path.last().cloned().unwrap_or(
            self.root.location.to_child_location(self.end.sub_path.root_entry)
        )
    }
}
impl LeafKey for ChildState {
    fn leaf_location(&self) -> ChildLocation {
        self.paths.leaf_location()
    }
}
impl LeafKey for PathPair {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
impl LeafKey for RangeEnd {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
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
impl GetCacheKey for ChildState {
    fn leaf_key<
            Trav: Traversable,
        >(&self, trav: &Trav) -> CacheKey {
        CacheKey::new(
            self.paths.path.role_leaf_child::<End, _>(trav),
            *self.query_pos()
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
            EndKind::Prefix(path) => CacheKey::new(path.root_parent(), self.root_pos),
            EndKind::Complete(c) => CacheKey::new(*c, self.root_pos),
        }
    }
}

pub trait TargetKey {
    fn target_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey;
}
impl TargetKey for TraversalState {
    fn target_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.leaf_key(trav),
            InnerKind::End(state) => state.target_key(trav),
        }
    }
}
impl TargetKey for EndState {
    fn target_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey {
        match &self.kind {
            EndKind::Range(state) => CacheKey::new(state.path.role_leaf_child::<End, _>(trav), *self.query_pos()),
            EndKind::Postfix(_) |
            EndKind::Prefix(_) => self.root_key(),
            EndKind::Complete(c) => CacheKey::new(*c, *self.query_pos()),
        }
    }
}