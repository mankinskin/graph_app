use context::*;

use hypercontext_api::{
    graph::{
        getters::ErrorReason, vertex::{
            child::Child,
            location::child::ChildLocation,
            pattern::IntoPattern,
        }, HypergraphRef
    }, path::structs::query_range_path::QueryRangePath
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
    ) -> Result<(Child, QueryRangePath), ErrorReason>;
}
impl HasInsertContext for HypergraphRef {
    fn indexer(&self) -> InsertContext {
        InsertContext::new(self.clone())
    }
    fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), ErrorReason> {
        self.indexer().insert_pattern(pattern)
    }
    //pub fn index_query_with_origin<
    //    Q: QueryPath
    //>(
    //    &self,
    //    query: Q,
    //) -> Result<(OriginPath<Child>, Q), ErrorReason> {
    //    self.indexer().index_query_with_origin(query)
    //}
}

#[derive(Debug, Clone)]
pub struct IndexSplitResult {
    pub inner: Child,
    pub location: ChildLocation,
    pub path: Vec<ChildLocation>,
}
