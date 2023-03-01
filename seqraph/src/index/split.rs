use crate::*;

#[derive(Debug, Clone)]
pub struct Splitter<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> {
    pub(crate) indexer: Indexer,
    _ty: std::marker::PhantomData<Side>,
}
impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Splitter<Side> {
    pub fn new(indexer: Indexer) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}

impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Splitter<Side> {
    pub fn pather(&self) -> Pather<Side> {
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