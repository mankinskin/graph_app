use std::num::NonZeroUsize;

use crate::*;
use super::*;


#[derive(Debug, Clone)]
pub struct Splitter<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    indexer: Indexer<T, D>,
    _ty: std::marker::PhantomData<(D, Side)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub fn new(indexer: Indexer<T, D>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> Traversable<'a, 'g, T> for Splitter<T, D, Side> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard {
        self.indexer.graph().await
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> TraversableMut<'a, 'g, T> for Splitter<T, D, Side> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.indexer.graph_mut().await
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub(crate) fn pather(&self) -> Pather<T, D, Side> {
        Pather::new(self.indexer.clone())
    }
    #[instrument(skip(self, path))]
    pub async fn single_path_split(
        &'a mut self,
        path: impl ContextPath,
    ) -> Option<IndexSplitResult> {
        self.pather().index_primary_path::<InnerSide, _>(
            path,
        ).await
    }
    #[instrument(skip(self, parent, offset))]
    pub async fn single_offset_split(
        &'a mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        self.pather().single_offset_split::<InnerSide>(
            parent,
            offset,
        ).await
    }
}