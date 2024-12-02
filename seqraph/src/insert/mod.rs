use context::*;

use crate::{
    graph::HypergraphRef,
    search::NoMatch,
    traversal::path::structs::query_range_path::QueryRangePath,
};
use crate::graph::vertex::{
    child::Child,
    location::child::ChildLocation,
    pattern::IntoPattern,
};

pub mod context;
pub mod side;

#[cfg(test)]
#[macro_use]
pub mod tests;

impl HypergraphRef {
    pub fn indexer(&self) -> InsertContext {
        InsertContext::new(self.clone())
    }
    pub fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexer().insert_pattern(pattern)
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
