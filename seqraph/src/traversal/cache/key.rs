use crate::*;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub root: usize,
    pub token_pos: usize,
}
pub trait GetCacheKey {
    fn cache_key(&self) -> CacheKey;
}
impl<R: ResultKind, Q: TraversalQuery> GetCacheKey for TraversalResult<R, Q> {
    fn cache_key(&self) -> CacheKey {
        CacheKey {
            root: self.found.location.parent.index(),
            token_pos: self.token_pos,
        }
    }
}
impl GetCacheKey for EndPath {
    fn cache_key(&self) -> CacheKey {
        CacheKey {
            root: self.entry.index(),
            token_pos: self.width,
        }
    }
}
impl GetCacheKey for StartLeaf {
    fn cache_key(&self) -> CacheKey {
        CacheKey {
            root: self.entry.index(),
            //sub_index: self.entry.sub_index,
            token_pos: self.token_pos,
        }
    }
}
impl GetCacheKey for StartPath {
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

impl<P: NewAdvanced, Q: TraversalQuery> GetCacheKey for PathPair<P, Q> {
    fn cache_key(&self) -> CacheKey {
        self.get_path().cache_key()
    }
}
impl<R: ResultKind, Q: TraversalQuery> GetCacheKey for ParentNode<R, Q> {
    fn cache_key(&self) -> CacheKey {
        self.path.cache_key()
    }
}