use std::ops::Deref;

use context::*;

use hypercontext_api::{
    graph::{
        getters::ErrorReason,
        vertex::child::Child,
        HypergraphRef,
    },
    interval::InitInterval,
    path::structs::rooted::pattern_range::PatternRangePath,
    traversal::{
        fold::{
            foldable::{
                ErrorState,
                Foldable,
            },
            state::FoldState,
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

pub trait ToInsertContext: TraversableMut {
    fn insert_context(&self) -> InsertContext;

    fn insert(
        &self,
        foldable: impl Foldable,
    ) -> Result<Child, ErrorState> {
        self.insert_context().insert(foldable)
    }
    fn insert_init(
        &self,
        init: InitInterval,
    ) -> Child {
        self.insert_context().insert_init(init)
    }
    fn insert_or_get_complete(
        &self,
        foldable: impl Foldable,
    ) -> Result<Child, ErrorReason> {
        self.insert_context().insert_or_get_complete(foldable)
    }
}
impl ToInsertContext for HypergraphRef {
    fn insert_context(&self) -> InsertContext {
        InsertContext::from(self.clone())
    }
}
impl<T: ToInsertContext> ToInsertContext for &'_ mut T {
    fn insert_context(&self) -> InsertContext {
        (**self).insert_context()
    }
}
