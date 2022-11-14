use crate::*;
mod reader;
mod overlap;
#[cfg(test)]
mod tests;

pub(crate) use {
    reader::*,
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
    pub async fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = T> + std::fmt::Debug + Send + Sync,
    ) -> Option<Child> {
        self.right_reader().read_sequence(sequence).await
    }
    pub async fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child> {
        self.right_reader().read_pattern(pattern).await
    }
}