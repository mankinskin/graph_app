use crate::*;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub index: Child,
    pub token_pos: usize,
}
impl CacheKey {
    pub fn new(index: Child, token_pos: usize) -> Self {
        Self {
            index,
            token_pos,
        }
    }
}
pub trait GetCacheKey: RootKey + LeafKey {
}
impl<T: RootKey + LeafKey> GetCacheKey for T {}

pub trait RootKey {
    fn root_key(&self) -> CacheKey;
}
pub trait LeafKey {
    fn leaf_location(&self) -> ChildLocation;
    fn leaf_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey {
        let c = trav.graph().expect_child_at(self.leaf_location());
        CacheKey {
            index: c,
            token_pos: 0,
        }
    }
}
impl LeafKey for SearchPath {
    fn leaf_location(&self) -> ChildLocation {
        self.end.path.last().cloned().unwrap_or(
            self.root.to_child_location(self.end.sub_path.root_entry)
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
        self.path().leaf_location()
    }
}
impl<
> LeafKey for RangeEnd {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}
impl<R: PathRole> RootKey for RootedRolePath<R> {
    fn root_key(&self) -> CacheKey {
        CacheKey {
            index: self.root_parent(),
            token_pos: 0,
        }
    }
}
impl RootKey for Child {
    fn root_key(&self) -> CacheKey {
        CacheKey {
            index: self.clone(),
            token_pos: 0,
        }
    }
}
impl RootKey for SearchPath {
    fn root_key(&self) -> CacheKey {
        self.root.parent.root_key()
    }
}
impl RootKey for PathPair {
    fn root_key(&self) -> CacheKey {
        self.path().root_key()
    }
}
impl RootKey for ParentState {
    fn root_key(&self) -> CacheKey {
        self.path.root_key()
    }
}
impl RootKey for StartState {
    fn root_key(&self) -> CacheKey {
        self.index.root_key()
    }
}
impl RootKey for ChildState {
    fn root_key(&self) -> CacheKey {
        self.paths.root_key()
    }
}
impl<P: MatchEndPath + RootKey> RootKey for MatchEnd<P> {
    fn root_key(&self) -> CacheKey {
        match self {
            Self::Path(path) => path.root_key(),
            Self::Complete(c) => c.root_key(),
        }
    }
}
impl<
> RootKey for TraversalState {
    fn root_key(&self) -> CacheKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.root_key(),
            InnerKind::End(state) => state.root_key(),
        }
    }
}
impl<
> RootKey for EndState {
    fn root_key(&self) -> CacheKey {
        match &self.kind {
            EndKind::Range(s) => s.path.root_key(),
            EndKind::Postfix(path) => path.root_key(),
            EndKind::Prefix(path) => path.root_key(),
            EndKind::Complete(c) => c.root_key(),
        }
    }
}
pub trait TargetKey {
    fn target_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey;
}

impl<
> TargetKey for TraversalState {
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
impl<
> TargetKey for EndState {
    fn target_key<
        Trav: Traversable,
    >(&self, trav: &Trav) -> CacheKey {
        match &self.kind {
            EndKind::Range(state) => state.leaf_key(trav),
            EndKind::Postfix(_) |
            EndKind::Prefix(_) => self.root_key(),
            EndKind::Complete(c) => c.root_key(),
        }
    }
}
//impl<R: ResultKind, Q: BaseQuery> GetCacheKey for TraversalResult<R, Q> {
//    fn leaf_key(&self) -> CacheKey {
//        self.path.leaf_key()
//    }
//    fn root_key(&self) -> CacheKey {
//        self.path.root_key()
//    }
//}
//impl GetCacheKey for RolePath {
//    fn cache_key(&self) -> CacheKey {
//        CacheKey {
//            root: self.entry.index(),
//            token_pos: self.width,
//        }
//    }
//}
//impl GetCacheKey for PathLeaf {
//    fn cache_key(&self) -> CacheKey {
//        CacheKey {
//            root: self.entry.index(),
//            //sub_index: self.entry.sub_index,
//            token_pos: self.token_pos,
//        }
//    }
//}
//impl<R: PathRole> LeafKey for RolePath<R> {
//    fn leaf_key(&self) -> CacheKey {
//        CacheKey {
//            index: self.child,
//            token_pos: 0,
//        }
//    }
//}
//impl<R: PathRole> RootKey for RolePath<R> {
//    fn root_key(&self) -> CacheKey {
//        CacheKey {
//            index: self.root_parent(),
//            token_pos: 0,
//        }
//    }
//}
//impl<R: PathRole> LeafKey for RootedRolePath<R> {
//    fn leaf_key(&self) -> CacheKey {
//        CacheKey {
//            index: self.child,
//            token_pos: 0,
//        }
//    }
//}
//impl GetCacheKey for FoundPath {
//    fn leaf_key(&self) -> CacheKey {
//        match self {
//            Self::Complete(c) => c.leaf_key(),
//            Self::Path(path) => path.leaf_key(),
//            Self::Postfix(path) => path.leaf_key(),
//            Self::Prefix(path) => path.leaf_key(),
//        }
//    }
//    fn root_key(&self) -> CacheKey {
//        match self {
//            Self::Complete(c) => c.root_key(),
//            Self::Path(path) => path.root_key(),
//            Self::Postfix(path) => path.root_key(),
//            Self::Prefix(path) => path.root_key(),
//        }
//    }
//}
//impl<P: GetCacheKey> LeafKey for OriginPath<P> {
//    fn leaf_key(&self) -> CacheKey {
//        self.postfix.leaf_key()
//    }
//}
//impl<P: GetCacheKey> RootKey for OriginPath<P> {
//    fn root_key(&self) -> CacheKey {
//        self.postfix.root_key()
//    }
//}
//impl<
//    R: ResultKind,
//> LeafKey for TraversalState<R> {
//    fn leaf_key(&self) -> CacheKey {
//        match self {
//            Self::Start(start) => CacheKey {
//                index: start.index,
//                token_pos: 0,
//            },
//            Self::Parent(_, node) =>
//                node.leaf_key(),
//            Self::Child(_, paths) =>
//                paths.leaf_key(),
//            Self::End(_, state) =>
//                state.leaf_key(),
//        }
//    }
//}
//impl<
//    R: ResultKind,
//> RootKey for TraversalState<R> {
//    fn root_key(&self) -> CacheKey {
//        match self {
//            Self::Start(node) => node.root_key(),
//            Self::Inner(state) => state.root_key()
//        }
//    }
//}
//impl<
//    R: ResultKind,
//> LeafKey for EndState<R> {
//    fn leaf_key(&self) -> CacheKey {
//        match self {
//            Self::Mismatch(_, _, _, leaf_key)
//            | Self::QueryEnd(_, _, _, leaf_key) =>
//                *leaf_key,
//            Self::MatchEnd(_, match_end, _) =>
//                match_end.leaf_key(),
//        }
//    }
//}
//impl<
//    R: ResultKind,
//> TargetKey for TraversalState<R> {
//    fn target_key(&self) -> CacheKey {
//        match self {
//            Self::Start(_)
//                => self.root_key(),
//            Self::Inner(state)
//                => state.target_key(),
//        }
//    }
//}