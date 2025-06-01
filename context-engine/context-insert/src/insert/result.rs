use std::{
    borrow::Borrow,
    fmt::Debug,
};

use crate::join::context::frontier::FrontierSplitIterator;

use context_trace::graph::vertex::child::Child;
pub trait InsertResult: Debug + Borrow<Child> + From<Child> {
    fn build_with_ctx(
        root: Child,
        _ctx: FrontierSplitIterator,
    ) -> Self {
        root.into()
    }
}
impl InsertResult for Child {}

#[derive(Debug)]
pub struct InsertResultWithPath {
    index: Child,
}
impl From<Child> for InsertResultWithPath {
    fn from(index: Child) -> Self {
        Self { index }
    }
}
impl Borrow<Child> for InsertResultWithPath {
    fn borrow(&self) -> &Child {
        &self.index
    }
}
impl InsertResult for InsertResultWithPath {
    fn build_with_ctx(
        root: Child,
        _ctx: FrontierSplitIterator,
    ) -> Self {
        root.into()
    }
}

//#[derive(Debug, Clone)]
//pub struct IndexSplitResult {
//    pub inner: Child,
//    pub location: ChildLocation,
//    pub path: Vec<ChildLocation>,
//}
