use crate::*;

mod indexer;
mod index_direction;
mod side;
mod context;
mod split;
mod join;
mod partition;
mod path;
mod merge;

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
pub use merge::*;

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
