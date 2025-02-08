use context::*;

use hypercontext_api::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            location::child::ChildLocation,
            pattern::IntoPattern,
        },
        HypergraphRef,
    },
    path::structs::rooted::pattern_range::PatternRangePath,
    traversal::{
        fold::state::FoldState,
        traversable::TraversableMut,
    },
};

pub mod context;
pub mod direction;

#[cfg(test)]
#[macro_use]
pub mod tests;

pub trait HasInsertContext: TraversableMut {
    fn insert_context(&self) -> InsertContext;
    fn insert_fold_state(
        &self,
        fold_state: FoldState,
    ) -> (Child, PatternRangePath) {
        self.insert_context().insert(fold_state)
    }
}
impl<T: HasInsertContext> HasInsertContext for &'_ mut T {
    fn insert_context(&self) -> InsertContext {
        (**self).insert_context()
    }
    fn insert_fold_state(
        &self,
        fold_state: FoldState,
    ) -> (Child, PatternRangePath) {
        (**self).insert_fold_state(fold_state)
    }
}
impl HasInsertContext for HypergraphRef {
    fn insert_context(&self) -> InsertContext {
        InsertContext::new(self.clone())
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
