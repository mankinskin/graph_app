use std::{
    borrow::Borrow,
    fmt::Debug,
};

use crate::join::context::frontier::FrontierSplitIterator;

use context_search::traversal::result::IncompleteState;
use context_trace::{
    graph::vertex::child::Child,
    path::structs::rooted::{
        pattern_range::PatternRangePath,
        role_path::PatternEndPath,
    },
};
pub trait ResultExtraction {
    fn extract_from(state: &IncompleteState) -> Self;
}
impl ResultExtraction for () {
    fn extract_from(_: &IncompleteState) -> Self {
        ()
    }
}
impl ResultExtraction for Option<PatternEndPath> {
    fn extract_from(state: &IncompleteState) -> Self {
        Some(state.end_state.cursor.path.clone())
    }
}
pub trait InsertResult: Debug + Borrow<Child> + From<Child> {
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
    pub path: Option<PatternEndPath>,
}
impl From<Child> for IndexWithPath {
    fn from(index: Child) -> Self {
        Self { index, path: None }
    }
}
impl Borrow<Child> for IndexWithPath {
    fn borrow(&self) -> &Child {
        &self.index
    }
}
impl InsertResult for IndexWithPath {
    type Extract = Option<PatternEndPath>;
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
