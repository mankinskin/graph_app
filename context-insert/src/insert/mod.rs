use context::*;

use crate::interval::InitInterval;
use context_search::{
    graph::{
        HypergraphRef,
        getters::ErrorReason,
        vertex::child::Child,
    },
    traversal::{
        fold::foldable::{
            ErrorState,
            Foldable,
        },
        traversable::TraversableMut,
    },
};

pub mod context;
pub mod direction;

//#[derive(Debug, Clone)]
//pub struct IndexSplitResult {
//    pub inner: Child,
//    pub location: ChildLocation,
//    pub path: Vec<ChildLocation>,
//}

pub trait ToInsertContext: TraversableMut
{
    fn insert_context(&self) -> InsertContext;

    fn insert(
        &self,
        foldable: impl Foldable,
    ) -> Result<Child, ErrorState>
    {
        self.insert_context().insert(foldable)
    }
    fn insert_init(
        &self,
        init: InitInterval,
    ) -> Child
    {
        self.insert_context().insert_init(init)
    }
    fn insert_or_get_complete(
        &self,
        foldable: impl Foldable,
    ) -> Result<Child, ErrorReason>
    {
        self.insert_context().insert_or_get_complete(foldable)
    }
}
impl ToInsertContext for HypergraphRef
{
    fn insert_context(&self) -> InsertContext
    {
        InsertContext::from(self.clone())
    }
}
impl<T: ToInsertContext> ToInsertContext for &'_ mut T
{
    fn insert_context(&self) -> InsertContext
    {
        (**self).insert_context()
    }
}
