use overlap::*;
use reader::*;
use sequence::*;

use crate::shared::*;

mod overlap;
mod reader;
pub mod sequence;
#[cfg(test)]
mod tests;

impl HypergraphRef {
    pub fn read_context<'g>(&'g self) -> ReadContext<'g> {
        ReadContext::new(self.graph_mut())
    }
    pub fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item=DefaultToken> + std::fmt::Debug + Send + Sync,
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
