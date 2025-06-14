use std::{
    borrow::Borrow,
    fmt::Debug,
};

use context_search::traversal::result::IncompleteState;
use context_trace::{
    graph::vertex::child::Child,
    path::structs::rooted::pattern_range::PatternRangePath,
};

pub trait ResultExtraction {
    fn extract_from(state: &IncompleteState) -> Self;
}
impl ResultExtraction for () {
    fn extract_from(_: &IncompleteState) -> Self {
        ()
    }
}
impl ResultExtraction for PatternRangePath {
    fn extract_from(state: &IncompleteState) -> Self {
        state.end_state.cursor.path.clone().into()
    }
}
pub trait TryInitWith<T>: Sized {
    type Error;
    fn try_init(value: T) -> Result<Self, Self::Error>;
}
//impl<T, A: TryFrom<T>> TryInitWith<T> for A {
//    type Error = <A as TryFrom<T>>::Error;
//    fn try_init(value: T) -> Result<Self, Self::Error> {
//        Self::try_from(value)
//    }
//}
impl TryInitWith<Child> for Child {
    type Error = Child;
    fn try_init(value: Child) -> Result<Self, Self::Error> {
        Ok(value)
    }
}
impl TryInitWith<Child> for IndexWithPath {
    type Error = Child;
    fn try_init(value: Child) -> Result<Self, Self::Error> {
        Err(value)
    }
}
pub trait InsertResult:
    Debug + Borrow<Child> + TryInitWith<Child, Error = Child> + Into<Child>
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

#[derive(Debug, Clone)]
pub struct IndexWithPath {
    pub index: Child,
    pub path: PatternRangePath,
}
impl TryFrom<Child> for IndexWithPath {
    type Error = Child;
    fn try_from(value: Child) -> Result<Self, Self::Error> {
        Err(value)
    }
}
impl Into<Child> for IndexWithPath {
    fn into(self) -> Child {
        self.index
    }
}
impl Borrow<Child> for IndexWithPath {
    fn borrow(&self) -> &Child {
        &self.index
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
