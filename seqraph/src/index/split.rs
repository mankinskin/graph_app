use crate::*;

#[derive(Debug, Clone)]
pub struct Splitter<G: GraphKind, Side: IndexSide<G::Direction>> {
    pub(crate) indexer: Indexer<G>,
    _ty: std::marker::PhantomData<(G, Side)>,
}
impl<G: GraphKind, Side: IndexSide<G::Direction>> Splitter<G, Side> {
    pub fn new(indexer: Indexer<G>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}

impl<G: GraphKind, Side: IndexSide<G::Direction>> Splitter<G, Side> {
    pub fn pather(&self) -> Pather<G, Side> {
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