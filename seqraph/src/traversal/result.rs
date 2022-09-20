use crate::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TraversalResult<Q: TraversalQuery> {
    pub(crate) found: FoundPath,
    pub(crate) query: Q,
}

impl<Q: TraversalQuery> TraversalResult<Q> {
    pub(crate) fn new(found: FoundPath, query: Q) -> Self {
        Self {
            found,
            query,
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
impl<Q: QueryPath> TraversalResult<Q> {
    #[allow(unused)]
    pub fn complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}