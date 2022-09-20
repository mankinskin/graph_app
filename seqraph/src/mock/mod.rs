use crate::*;

//pub mod gen_graph;
//pub use gen_graph::*;

#[allow(unused)]
pub fn pattern_from_widths(widths: impl IntoIterator<Item=usize>) -> Pattern {
    widths.into_iter()
        .enumerate()
        .map(|(i, w)| Child::new(i, w))
        .collect()
}