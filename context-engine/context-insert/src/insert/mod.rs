use context::*;
use result::InsertResult;

use crate::interval::init::InitInterval;
use context_search::traversal::fold::foldable::{
    ErrorState,
    Foldable,
};
use context_trace::{
    graph::{
        HypergraphRef,
        getters::ErrorReason,
    },
    trace::has_graph::HasGraphMut,
};
pub mod context;
pub mod direction;
pub mod result;

pub trait ToInsertContext<R: InsertResult>: HasGraphMut {
    fn insert_context(&self) -> InsertContext<R>;

    fn insert(
        &self,
        foldable: impl Foldable,
    ) -> Result<R, ErrorState> {
        self.insert_context().insert(foldable)
    }
    fn insert_init(
        &self,
        init: InitInterval,
    ) -> R {
        self.insert_context().insert_init(init)
    }
    fn insert_or_get_complete(
        &self,
        foldable: impl Foldable,
    ) -> Result<R, ErrorReason> {
        self.insert_context().insert_or_get_complete(foldable)
    }
}
impl<R: InsertResult> ToInsertContext<R> for HypergraphRef {
    fn insert_context(&self) -> InsertContext<R> {
        InsertContext::<R>::from(self.clone())
    }
}
impl<R: InsertResult, T: ToInsertContext<R>> ToInsertContext<R> for &'_ mut T {
    fn insert_context(&self) -> InsertContext<R> {
        (**self).insert_context()
    }
}
