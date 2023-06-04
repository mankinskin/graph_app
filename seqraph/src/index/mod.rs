use crate::*;

pub mod indexer;
pub mod index_direction;
pub mod side;
pub mod context;
pub mod split;
pub mod join;
pub mod partition;
pub mod path;

#[cfg(test)]
#[macro_use]
pub mod tests;


pub use indexer::*;
pub use index_direction::*;
pub use side::*;
pub use split::*;
pub use context::*;
pub use path::*;
pub use partition::*;
pub use join::*;

impl<'t, 'g> HypergraphRef {
    pub fn indexer(&self) -> Indexer {
        Indexer::new(self.clone())
    }
    pub fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexer().index_pattern(pattern)
    }
    //pub fn index_query_with_origin<
    //    Q: QueryPath
    //>(
    //    &self,
    //    query: Q,
    //) -> Result<(OriginPath<Child>, Q), NoMatch> {
    //    self.indexer().index_query_with_origin(query)
    //}
}

#[derive(Debug, Clone)]
pub struct IndexSplitResult {
    pub inner: Child,
    pub location: ChildLocation,
    pub path: Vec<ChildLocation>,
}
