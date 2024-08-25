use crate::traversal::{
    folder::state::FoldResult,
    path::{
        accessors::complete::PathComplete,
        structs::query_range_path::{
            QueryPath,
            QueryRangePath,
        },
    },
};
use crate::graph::vertex::{
    child::Child,
    pattern::IntoPattern,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraversalResult {
    pub result: FoldResult,
    pub query: QueryRangePath,
}

impl TraversalResult {
    pub fn new(
        result: impl Into<FoldResult>,
        query: impl Into<QueryRangePath>,
    ) -> Self {
        Self {
            result: result.into(),
            query: query.into(),
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.result.unwrap_complete()
    }
    #[allow(unused)]
    #[track_caller]
    pub fn expect_complete(
        self,
        msg: &str,
    ) -> Child {
        self.result.expect_complete(msg)
    }
}

impl TraversalResult {
    #[allow(unused)]
    pub fn new_complete(
        query: impl IntoPattern,
        index: impl crate::graph::vertex::has_vertex_index::ToChild,
    ) -> Self {
        Self {
            result: FoldResult::Complete(index.to_child()),
            query: QueryRangePath::complete(query),
        }
    }
}
