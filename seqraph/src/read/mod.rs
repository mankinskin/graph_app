use crate::{
    insert::HasInsertContext,
    read::reader::context::ReadContext,
};
use hypercontext_api::graph::{
    vertex::{
        child::Child,
        pattern::IntoPattern,
    },
    HypergraphRef,
};
use sequence::ToNewTokenIndices;

pub mod bundle;
pub mod overlap;
pub mod reader;
pub mod sequence;
//#[cfg(test)]
//mod tests;

pub trait HasReadContext: HasInsertContext {
    fn read_context<'g>(&'g mut self) -> ReadContext;
    fn read_sequence(
        &mut self,
        //sequence: impl IntoIterator<Item = DefaultToken> + std::fmt::Debug + Send + Sync,
        sequence: impl ToNewTokenIndices,
    ) -> Option<Child>;
    fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child>;
}

impl HasReadContext for HypergraphRef {
    fn read_context(&mut self) -> ReadContext {
        //ReadContext::new(self.graph_mut())
        ReadContext::new(self.clone())
    }
    fn read_sequence(
        &mut self,
        //sequence: impl IntoIterator<Item = DefaultToken> + std::fmt::Debug + Send + Sync,
        sequence: impl ToNewTokenIndices,
    ) -> Option<Child> {
        self.read_context().read_sequence(sequence)
    }
    fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child> {
        self.read_context().read_pattern(pattern)
    }
}
