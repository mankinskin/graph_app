use crate::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraversalResult<R: ResultKind, Q: TraversalQuery> {
    pub path: <R as ResultKind>::Found,
    pub query: Q,
}

pub trait IntoResult<R: ResultKind, Q: TraversalQuery>: RangePath {
    fn into_result(self, query: Q) -> TraversalResult<R, Q>;
}

impl<R: ResultKind, Q: TraversalQuery> IntoResult<R, Q> for <R as ResultKind>::Found {
    fn into_result(self, query: Q) -> TraversalResult<R, Q> {
        TraversalResult {
            path: self,
            query,
        }
    }
}
impl<R: ResultKind, Q: TraversalQuery> TraversalResult<R, Q> {
    pub fn new(found: impl Into<<R as ResultKind>::Found>, query: impl Into<Q>) -> Self {
        Self {
            path: found.into(),
            query: query.into(),
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.path.unwrap_complete()
    }
    #[allow(unused)]
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        self.path.expect_complete(msg)
    }
}
impl<Q: QueryPath> TraversalResult<BaseResult, Q> {
    #[allow(unused)]
    pub fn new_complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            path: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}