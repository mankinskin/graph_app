pub mod indexer;
use indexer::*;

pub mod side;
use crate::{
    graph::HypergraphRef,
    search::NoMatch,
    traversal::path::structs::query_range_path::QueryRangePath,
    vertex::{
        child::Child,
        location::ChildLocation,
        pattern::IntoPattern,
    },
};

//pub mod context;
//pub use context::*;

//pub mod path;
//pub use path::*;

#[cfg(test)]
#[macro_use]
pub mod tests;

impl<'t, 'g> HypergraphRef {
    pub fn indexer(&self) -> Indexer {
        Indexer::new(self.clone())
    }
    pub fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexer().index_pattern(pattern)
    }
    //pub fn index_query_with_origin<
    //    Q: QueryPath
    //>(
    //    &self,
    //    query: Q,
    //) -> Result<(OriginPath<Child>, Q), NoMatch> {
    //    self.indexer().index_query_with_origin(query)
    //}
}

#[derive(Debug, Clone)]
pub struct IndexSplitResult {
    pub inner: Child,
    pub location: ChildLocation,
    pub path: Vec<ChildLocation>,
}
