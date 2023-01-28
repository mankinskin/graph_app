use crate::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraversalResult<R: ResultKind> {
    pub path: <R as ResultKind>::Found,
    pub query: <R as ResultKind>::Query,
}

impl<R: ResultKind> TraversalResult<R> {
    pub fn new(
        found: impl Into<<R as ResultKind>::Found>,
        query: impl Into<<R as ResultKind>::Query>,
    ) -> Self {
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
impl TraversalResult<BaseResult> {
    #[allow(unused)]
    pub fn new_complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            path: FoundPath::Complete(index.as_child()),
            query: QueryRangePath::complete(query),
        }
    }
}