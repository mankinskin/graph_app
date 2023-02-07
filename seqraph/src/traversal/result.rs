use crate::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraversalResult {
    pub path: FoundPath,
    pub query: QueryRangePath,
}

impl TraversalResult {
    pub fn new(
        found: impl Into<FoundPath>,
        query: impl Into<QueryRangePath>,
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
impl TraversalResult {
    #[allow(unused)]
    pub fn new_complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            path: FoundPath::Complete(index.as_child()),
            query: QueryRangePath::complete(query),
        }
    }
}