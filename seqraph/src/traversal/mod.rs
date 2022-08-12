pub(crate) mod bft;
pub(crate) mod dft;
pub(crate) mod path;
pub(crate) mod node;
pub(crate) mod traversable;
pub(crate) mod folder;
pub(crate) mod iterator;
pub(crate) mod policy;
pub(crate) mod match_end;
pub(crate) mod cache;
pub(crate) mod found_path;

pub(crate) use super::*;
pub(crate) use bft::*;
pub(crate) use dft::*;
pub(crate) use path::*;
pub(crate) use node::*;
pub(crate) use traversable::*;
pub(crate) use folder::*;
pub(crate) use iterator::*;
pub(crate) use policy::*;
pub(crate) use match_end::*;
pub(crate) use cache::*;
pub(crate) use found_path::*;

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