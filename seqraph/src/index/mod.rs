use crate::{
    vertex::*,
    search::*,
    HypergraphRef, QueryRangePath,
};
use std::ops::RangeFrom;

mod indexer;
mod index_direction;
mod side;
mod context;
mod split;
mod indexing;
mod origin_path;
mod path;

#[cfg(test)]
#[macro_use]
pub(crate) mod tests;


pub use indexer::*;
pub use index_direction::*;
pub(crate) use side::*;
pub(crate) use split::*;
pub(crate) use context::*;
pub(crate) use origin_path::*;
pub(crate) use path::*;

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
    pub(crate) fn index_query_with_origin<
        Q: IndexingQuery
    >(
        &self,
        query: Q,
    ) -> Result<(OriginPath<Child>, Q), NoMatch> {
        self.indexer().index_query_with_origin(query)
    }
}

#[derive(Debug, Clone)]
pub struct IndexSplitResult {
    pub(crate) inner: Child,
    pub(crate) location: ChildLocation,
    pub(crate) path: Vec<ChildLocation>,
}
