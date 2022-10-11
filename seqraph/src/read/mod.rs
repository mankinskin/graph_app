use crate::{
    direction::*,
    vertex::*,
    HypergraphRef,
};
mod reader;
mod overlap_bands;
mod overlaps;
#[cfg(test)]
mod tests;

pub use {
    reader::*,
    overlap_bands::*,
};
//mod async_reader;
//pub use async_reader::*;

impl<T: Tokenize + Send> HypergraphRef<T> {
    pub fn right_reader(&self) -> Reader<T, Right> {
        Reader::new(self.clone())
    }
    pub fn left_reader(&self) -> Reader<T, Left> {
        Reader::new(self.clone())
    }
    pub fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> Child {
        self.right_reader().read_sequence(sequence)
    }
    pub fn read_pattern(
        &mut self,
        pattern: Pattern,
    ) -> Child {
        self.right_reader().read_pattern(pattern)
    }
}