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
    fn insert_context(&self) -> InsertContext;
    fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), ErrorReason>;
}
impl HasInsertContext for HypergraphRef {
    fn insert_context(&self) -> InsertContext {
        InsertContext::new(self.clone())
    }
    fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), ErrorReason> {
        self.insert_context().insert(pattern.into_pattern())
    }
    //pub fn index_query_with_origin<
    //    Q: QueryPath
    //>(
    //    &self,
    //    query: Q,
    //) -> Result<(OriginPath<Child>, Q), ErrorReason> {
    //    self.insert_context().index_query_with_origin(query)
    //}
}

#[derive(Debug, Clone)]
pub struct IndexSplitResult {
    pub inner: Child,
    pub location: ChildLocation,
    pub path: Vec<ChildLocation>,
}
