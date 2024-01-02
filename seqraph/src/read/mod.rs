pub use crate::shared::*;
mod reader;
mod overlap;
#[cfg(test)]
mod tests;
pub mod sequence;

pub use {
    reader::*,
    overlap::*,
    sequence::*,
};

impl HypergraphRef {
    pub fn read_context<'g>(&'g self) -> ReadContext<'g> {
        ReadContext::new(self.graph_mut())
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