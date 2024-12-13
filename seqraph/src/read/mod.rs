
use hypercontext_api::{
    graph::{
        kind::DefaultToken,
        vertex::{
            child::Child,
            pattern::IntoPattern,
        },
        HypergraphRef,
    },
    traversal::traversable::TraversableMut,
};
use crate::read::reader::context::ReadContext;

pub mod reader;
pub mod sequence;
pub mod bands;
#[cfg(test)]
mod tests;

impl HypergraphRef {
    pub fn read_context<'g>(&'g mut self) -> ReadContext<'g> {
        //ReadContext::new(self.graph_mut())
        ReadContext::new(self.clone())
    }
    pub fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = DefaultToken> + std::fmt::Debug + Send + Sync,
    ) -> Option<Child> {
        self.read_context().read_sequence(sequence)
    }
    pub fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child> {
        self.read_context().read_pattern(pattern)
    }
}
