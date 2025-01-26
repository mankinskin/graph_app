use std::num::NonZeroUsize;

use hypercontext_api::graph::vertex::child::Child;

use crate::insert::IndexSplitResult;

use super::{context::ContextPath, side::{relative::InnerSide, IndexSide}};


#[derive(Debug, Clone)]
pub struct Splitter<Side: IndexSide> {
    pub(crate) indexer: Indexer,
    _ty: std::marker::PhantomData<Side>,
}

impl<Side: IndexSide> Splitter<Side> {
    pub fn new(indexer: Indexer) -> Self {
        Self {
            indexer,
            _ty: Default::default(),
        }
    }
}

impl<Side: IndexSide> Splitter<Side> {
    pub fn pather(&self) -> Pather<Side> {
        Pather::new(self.indexer.clone())
    }
    pub fn single_path_split(
        &mut self,
        path: impl ContextPath,
    ) -> Option<IndexSplitResult> {
        self.pather().index_primary_path::<InnerSide, _>(path)
    }
    pub fn single_offset_split(
        &mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        self.pather()
            .single_offset_split::<InnerSide>(parent, offset)
    }
}
