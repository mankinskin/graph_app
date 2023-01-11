use crate::*;

#[derive(Debug, Clone)]
pub struct Splitter<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    pub(crate) indexer: Indexer<T, D>,
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

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub fn pather(&self) -> Pather<T, D, Side> {
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