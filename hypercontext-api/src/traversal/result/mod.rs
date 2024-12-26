use crate::{
    path::{
        accessors::complete::PathComplete,
        structs::query_range_path::{
            QueryPath, QueryRangePath,
        },
    }, traversal::fold::state::FoldResult
};
use crate::graph::vertex::{
    child::Child,
    pattern::IntoPattern,
};

use super::state::query::QueryState;
pub mod kind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraversalResult {
    pub result: FoldResult,
    pub query: QueryState,
}

impl TraversalResult {
    pub fn new(
        result: impl Into<FoldResult>,
        query: impl Into<QueryState>,
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
        let query = query.into_pattern();
        Self {
            result: FoldResult::Complete(index.to_child()),
            query: QueryState {
                pos: query.len().into(),
                path: QueryRangePath::complete(query),
            }
        }
    }
}
