use crate::*;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub index: usize,
    pub token_pos: usize,
}
impl CacheKey {
    pub fn new(index: usize, token_pos: usize) -> Self {
        Self {
            index,
            token_pos,
        }
    }
}
pub trait GetCacheKey {
    fn leaf_key(&self) -> CacheKey;
    fn root_key(&self) -> CacheKey;
}
impl<R: ResultKind, Q: BaseQuery> GetCacheKey for TraversalResult<R, Q> {
    fn leaf_key(&self) -> CacheKey {
        self.path.leaf_key()
    }
    fn root_key(&self) -> CacheKey {
        self.path.root_key()
    }
}
//impl GetCacheKey for ChildPath {
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
impl<R: PathRole> GetCacheKey for ChildPath<R> {
    fn leaf_key(&self) -> CacheKey {
        CacheKey {
            index: self.path_child_location().parent.index(),
            token_pos: self.token_pos,
        }
    }
    fn root_key(&self) -> CacheKey {
        CacheKey {
            index: self.root_parent().index(),
            token_pos: self.token_pos,
        }
    }
}
impl GetCacheKey for Child {
    fn leaf_key(&self) -> CacheKey {
        CacheKey {
            index: self.index,
            token_pos: 0,
        }
    }
    fn root_key(&self) -> CacheKey {
        self.leaf_key()
    }
}
impl GetCacheKey for SearchPath {
    fn leaf_key(&self) -> CacheKey {
        self.start.leaf_key()
    }
    fn root_key(&self) -> CacheKey {
        // todo: use start or end?
        self.start.root_key()
    }
}

impl<P: Advanced, Q: BaseQuery> GetCacheKey for PathPair<P, Q> {
    fn leaf_key(&self) -> CacheKey {
        self.get_path().leaf_key()
    }
    fn root_key(&self) -> CacheKey {
        self.get_path().root_key()
    }
}
impl<R: ResultKind, Q: BaseQuery> GetCacheKey for ParentNode<R, Q> {
    fn leaf_key(&self) -> CacheKey {
        self.path.leaf_key()
    }
    fn root_key(&self) -> CacheKey {
        self.path.root_key()
    }
}
impl<
    R: ResultKind,
    Q: BaseQuery,
> GetCacheKey for StartNode<R, Q> {
    fn leaf_key(&self) -> CacheKey {
        CacheKey::new(self.index.index(), 0)
    }
    fn root_key(&self) -> CacheKey {
        self.leaf_key()
    }
}
impl<R: ResultKind, Q: BaseQuery> GetCacheKey for ChildNode<R, Q> {
    fn leaf_key(&self) -> CacheKey {
        self.paths.leaf_key()
    }
    fn root_key(&self) -> CacheKey {
        self.paths.root_key()
    }
}
impl GetCacheKey for FoundPath {
    fn leaf_key(&self) -> CacheKey {
        match self {
            Self::Complete(c) => c.leaf_key(),
            Self::Range(path) => path.leaf_key(),
            Self::Postfix(path) => path.leaf_key(),
            Self::Prefix(path) => path.leaf_key(),
        }
    }
    fn root_key(&self) -> CacheKey {
        match self {
            Self::Complete(c) => c.root_key(),
            Self::Range(path) => path.root_key(),
            Self::Postfix(path) => path.root_key(),
            Self::Prefix(path) => path.root_key(),
        }
    }
}
impl<P: MatchEndPath + GetCacheKey> GetCacheKey for MatchEnd<P> {
    fn leaf_key(&self) -> CacheKey {
        match self {
            MatchEnd::Path(path) => path.leaf_key(),
            MatchEnd::Complete(c) => c.leaf_key(),
        }
    }
    fn root_key(&self) -> CacheKey {
        match self {
            MatchEnd::Path(path) => path.root_key(),
            MatchEnd::Complete(c) => c.root_key(),
        }
    }
}
impl<P: GetCacheKey> GetCacheKey for OriginPath<P> {
    fn leaf_key(&self) -> CacheKey {
        self.postfix.leaf_key()
    }
    fn root_key(&self) -> CacheKey {
        self.postfix.root_key()
    }
}
impl<
    R: ResultKind,
    Q: BaseQuery,
> GetCacheKey for TraversalNode<R, Q> {
    fn leaf_key(&self) -> CacheKey {
        match self {
            TraversalNode::Start(start) => CacheKey {
                index: start.index.index(),
                token_pos: 0,
            },
            TraversalNode::Parent(_, node) =>
                node.leaf_key(),
            TraversalNode::Child(_, paths) =>
                paths.leaf_key(),
            TraversalNode::Mismatch(_, _, found) =>
                found.leaf_key(),
            TraversalNode::QueryEnd(_, _, found) =>
                found.leaf_key(),
            TraversalNode::MatchEnd(_, _, match_end, _) =>
                match_end.leaf_key(),
            //TraversalNode::Match(path, query) =>
            //    path.cache_key(),
        }
    }
    fn root_key(&self) -> CacheKey {
        match self {
            TraversalNode::Start(node) => node.root_key(),
            TraversalNode::Parent(_, node) => node.root_key(),
            TraversalNode::Child(_, paths) =>
                paths.root_key(),
            TraversalNode::Mismatch(_, _, found) =>
                found.root_key(),
            TraversalNode::QueryEnd(_, _, found) =>
                found.root_key(),
            TraversalNode::MatchEnd(_, _, match_end, _) =>
                match_end.root_key(),
            //TraversalNode::Match(path, query) =>
            //    path.cache_key(),
        }
    }
}