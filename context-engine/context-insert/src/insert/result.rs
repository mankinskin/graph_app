use std::{
    borrow::Borrow,
    fmt::Debug,
};

use context_search::*;
use context_trace::*;

pub trait ResultExtraction {
    fn extract_from(state: &IncompleteState) -> Self;
}
impl ResultExtraction for () {
    fn extract_from(_: &IncompleteState) -> Self {}
}
impl ResultExtraction for PatternRangePath {
    fn extract_from(state: &IncompleteState) -> Self {
        state.end_state.cursor.path.clone().into()
    }
}
pub trait TryInitWith<T>: Sized {
    type Error: Into<ErrorState>;
    fn try_init(value: T) -> Result<Self, Self::Error>;
}
//impl<T, A: TryFrom<T>> TryInitWith<T> for A {
//    type Error = <A as TryFrom<T>>::Error;
//    fn try_init(value: T) -> Result<Self, Self::Error> {
//        Self::try_from(value)
//    }
//}
impl TryInitWith<IndexWithPath> for Child {
    type Error = IndexWithPath;
    fn try_init(value: IndexWithPath) -> Result<Self, Self::Error> {
        Ok(value.index)
    }
}
impl TryInitWith<IndexWithPath> for IndexWithPath {
    type Error = IndexWithPath;
    fn try_init(value: IndexWithPath) -> Result<Self, Self::Error> {
        Err(value)
    }
}
//impl TryInitWith<ErrorState> for Child {
//    type Error = Child;
//    fn try_init(value: ErrorState) -> Result<Self, Self::Error> {
//        match value {
//            ErrorState {
//                reason: ErrorReason::SingleIndex(c),
//                found: Some(FinishedKind::Complete(_)),
//            } => Ok(c),
//            ErrorState {
//                reason: ErrorReason::SingleIndex(c),
//                found: Some(FinishedKind::Complete(_)),
//            } => Ok(c),
//        }
//    }
//}
//impl TryInitWith<ErrorState> for IndexWithPath {
//    type Error = Child;
//    fn try_init(value: ErrorState) -> Result<Self, Self::Error> {
//        Err(value)
//    }
//}
pub trait InsertResult:
    Debug
    + Borrow<Child>
    + TryInitWith<IndexWithPath>
    //+ TryInitWith<ErrorState, Error = Child>
    + Into<Child>
{
    type Extract: ResultExtraction;
    fn build_with_extract(
        root: Child,
        ext: Self::Extract,
    ) -> Self;
}
impl InsertResult for Child {
    type Extract = ();
    fn build_with_extract(
        root: Child,
        _: Self::Extract,
    ) -> Self {
        root
    }
}
impl InsertResult for IndexWithPath {
    type Extract = PatternRangePath;
    fn build_with_extract(
        root: Child,
        ext: Self::Extract,
    ) -> Self {
        Self {
            index: root,
            path: ext,
        }
    }
}

//#[derive(Debug, Clone)]
//pub struct IndexSplitResult {
//    pub inner: Child,
//    pub location: ChildLocation,
//    pub path: Vec<ChildLocation>,
//}
