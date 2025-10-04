use context::*;
use result::InsertResult;

use crate::interval::init::InitInterval;
use context_search::*;
use context_trace::*;
pub mod context;
pub mod direction;
pub mod result;

pub trait ToInsertCtx<R: InsertResult = Child>: HasGraphMut {
    fn insert_context(&self) -> InsertCtx<R>;

    fn insert(
        &self,
        foldable: impl Foldable,
    ) -> Result<R, ErrorState> {
        self.insert_context().insert(foldable)
    }
    fn insert_init(
        &self,
        ext: R::Extract,
        init: InitInterval,
    ) -> R {
        self.insert_context().insert_init(ext, init)
    }
    fn insert_or_get_complete(
        &self,
        foldable: impl Foldable,
    ) -> Result<Result<R, R::Error>, ErrorReason> {
        self.insert_context().insert_or_get_complete(foldable)
    }
}
impl<R: InsertResult> ToInsertCtx<R> for HypergraphRef {
    fn insert_context(&self) -> InsertCtx<R> {
        InsertCtx::<R>::from(self.clone())
    }
}
impl<R: InsertResult, T: ToInsertCtx<R>> ToInsertCtx<R> for &'_ mut T {
    fn insert_context(&self) -> InsertCtx<R> {
        (**self).insert_context()
    }
}
