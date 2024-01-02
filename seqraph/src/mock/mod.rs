use crate::shared::*;

//pub mod gen_graph;
//pub use gen_graph::*;

#[cfg(test)]
mod tests;

#[allow(unused)]
pub fn pattern_from_widths(widths: impl IntoIterator<Item=usize>) -> Pattern {
    widths.into_iter()
        .enumerate()
        .map(|(i, w)| Child::new(i, w))
        .collect()
}