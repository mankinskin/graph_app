use std::num::NonZeroUsize;

use crate::*;
use super::*;


#[derive(Debug, Clone)]
pub struct Splitter<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    indexer: Indexer<T, D>,
    _ty: std::marker::PhantomData<(D, Side)>,
}
impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub fn new(indexer: Indexer<T, D>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable<T> for Splitter<T, D, Side> {
    type Guard<'g> = RwLockReadGuard<'g, Hypergraph<T>> where Side: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self.indexer.graph()
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Splitter<T, D, Side> {
    type GuardMut<'g> = RwLockWriteGuard<'g, Hypergraph<T>> where Side: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g> {
        self.indexer.graph_mut()
    }
}
impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub(crate) fn pather(&self) -> Pather<T, D, Side> {
        Pather::new(self.indexer.clone())
    }
    #[instrument(skip(self, path))]
    pub fn single_path_split(
        &mut self,
        path: impl ContextPath,
    ) -> Option<IndexSplitResult> {
        self.pather().index_primary_path::<InnerSide, _>(
            path,
        )
    }
    #[instrument(skip(self, parent, offset))]
    pub fn single_offset_split(
        &mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        self.pather().single_offset_split::<InnerSide>(
            parent,
            offset,
        )
    }
}