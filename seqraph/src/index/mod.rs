use crate::{
    vertex::*,
    search::*,
    r#match::*,
    Hypergraph, ChildLocation,
};

mod indexer;
use indexer::*;
mod index_direction;
pub use index_direction::*;

impl<'t, 'g, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn indexer(&'g mut self) -> Indexer<'g, T> {
        Indexer::new(self)
    }
    pub(crate) fn index_found(
        &mut self,
        found_path: FoundPath,
    ) -> (Option<Child>, Child, Option<Child>, Pattern) {
        self.indexer().index_found(found_path)
    }
    /// does not include location
    pub(crate) fn index_pre_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.indexer().index_pre_context_at(location)
    }
    /// does not include location
    pub(crate) fn index_post_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.indexer().index_post_context_at(location)
    }
}