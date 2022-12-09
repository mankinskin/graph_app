use crate::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraversalResult<P: RangePath, Q: TraversalQuery> {
    pub found: P,
    pub query: Q,
}

pub trait IntoResult<Q: TraversalQuery>: RangePath {
    fn into_result(self, query: Q) -> TraversalResult<Self, Q>;
}

impl<P: RangePath, Q: TraversalQuery> IntoResult<Q> for P {
    fn into_result(self, query: Q) -> TraversalResult<Self, Q> {
        TraversalResult {
            found: self,
            query,
        }
    }
}
impl<F: RangePath, Q: TraversalQuery> TraversalResult<F, Q> {
    pub fn new(found: impl Into<F>, query: impl Into<Q>) -> Self {
        Self {
            found: found.into(),
            query: query.into(),
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.found.unwrap_complete()
    }
    #[allow(unused)]
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        self.found.expect_complete(msg)
    }
}
impl<Q: QueryPath> TraversalResult<FoundPath, Q> {
    #[allow(unused)]
    pub fn new_complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}