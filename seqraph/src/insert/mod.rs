use context::*;

use hypercontext_api::{
    graph::{
        vertex::child::Child,
        HypergraphRef,
    },
    path::structs::rooted::pattern_range::PatternRangePath,
    traversal::{
        fold::{
            state::FoldState,
            ErrorState,
            Foldable,
        },
        traversable::TraversableMut,
    },
};

pub mod context;
pub mod direction;

#[cfg(test)]
#[macro_use]
pub mod tests;

//#[derive(Debug, Clone)]
//pub struct IndexSplitResult {
//    pub inner: Child,
//    pub location: ChildLocation,
//    pub path: Vec<ChildLocation>,
//}

pub trait Inserting: TraversableMut {
    fn insert_context(&self) -> InsertContext;

    fn insert(
        &self,
        foldable: impl Foldable,
    ) -> Result<(Child, PatternRangePath), ErrorState> {
        self.insert_context().insert(foldable)
    }
    fn insert_state(
        &self,
        fold_state: FoldState,
    ) -> (Child, PatternRangePath) {
        self.insert_context().insert_state(fold_state)
    }
}
impl Inserting for HypergraphRef {
    fn insert_context(&self) -> InsertContext {
        InsertContext::new(self.clone())
    }
}
impl<T: Inserting> Inserting for &'_ mut T {
    fn insert_context(&self) -> InsertContext {
        (**self).insert_context()
    }
    fn insert(
        &self,
        foldable: impl Foldable,
    ) -> Result<(Child, PatternRangePath), ErrorState> {
        (**self).insert(foldable)
    }
    fn insert_state(
        &self,
        fold_state: FoldState,
    ) -> (Child, PatternRangePath) {
        (**self).insert_state(fold_state)
    }
}
