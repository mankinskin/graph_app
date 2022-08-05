use crate::{
    vertex::*,
    search::*,
    HypergraphRef, QueryRangePath,
};
use std::ops::RangeFrom;

mod indexer;
mod index_direction;
mod side;
mod side_indexable;

#[cfg(test)]
#[macro_use]
pub(crate) mod tests;


pub use indexer::*;
pub use index_direction::*;
pub(crate) use side::*;
pub(crate) use side_indexable::*;

impl<'t, 'g, T> HypergraphRef<T>
where
    T: Tokenize + 't,
{
    pub fn indexer(&self) -> Indexer<T, Right> {
        Indexer::new(self.clone())
    }
    pub fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexer().index_pattern(pattern)
    }
    pub(crate) fn index_query<
        Q: IndexingQuery
    >(
        &self,
        query: Q,
    ) -> Result<(Child, Q), NoMatch> {
        self.indexer().index_query(query)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IndexSplitResult {
    inner: Child,
    location: ChildLocation,
    path: Vec<ChildLocation>,
}
