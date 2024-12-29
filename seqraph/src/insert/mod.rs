use context::*;

use hypercontext_api::{
    path::structs::query_range_path::QueryRangePath,
    graph::{
        HypergraphRef,
        getters::NoMatch,
        vertex::{
            child::Child,
            location::child::ChildLocation,
            pattern::IntoPattern,
        },
    },
};

pub mod context;

#[cfg(test)]
#[macro_use]
pub mod tests;

pub trait HasInsertContext {
    fn indexer(&self) -> InsertContext;
    fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch>;
}
impl HasInsertContext for HypergraphRef {
    fn indexer(&self) -> InsertContext {
        InsertContext::new(self.clone())
    }
    fn index_pattern(
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
