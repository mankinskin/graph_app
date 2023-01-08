use crate::*;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub root: usize,
    pub token_pos: usize,
}
impl CacheKey {
    pub fn new(root: usize, token_pos: usize) -> Self {
        Self {
            root,
            token_pos,
        }
    }
}
pub trait GetCacheKey {
    fn cache_key(&self) -> CacheKey;
}
impl<R: ResultKind, Q: BaseQuery> GetCacheKey for TraversalResult<R, Q> {
    fn cache_key(&self) -> CacheKey {
        self.path.cache_key()
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
impl<R> GetCacheKey for ChildPath<R> {
    fn cache_key(&self) -> CacheKey {
        match self {
            Self::Leaf(leaf) => leaf.cache_key(),
            Self::Path { entry, token_pos, .. } => CacheKey {
                root: entry.index(),
                //sub_index: entry.sub_index,
                token_pos: *token_pos,
            },
        }
    }
}
impl GetCacheKey for Child {
    fn cache_key(&self) -> CacheKey {
        CacheKey {
            root: self.index,
            token_pos: 0,
        }
    }
}
impl GetCacheKey for SearchPath {
    fn cache_key(&self) -> CacheKey {
        self.start.cache_key()
    }
}

impl<P: Advanced, Q: BaseQuery> GetCacheKey for PathPair<P, Q> {
    fn cache_key(&self) -> CacheKey {
        self.get_path().cache_key()
    }
}
impl<R: ResultKind, Q: BaseQuery> GetCacheKey for ParentNode<R, Q> {
    fn cache_key(&self) -> CacheKey {
        self.path.cache_key()
    }
}
impl<R: ResultKind, Q: BaseQuery> GetCacheKey for ChildNode<R, Q> {
    fn cache_key(&self) -> CacheKey {
        self.paths.cache_key()
    }
}