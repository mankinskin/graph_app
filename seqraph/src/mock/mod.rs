//pub mod gen_graph;
//use gen_graph::*;

use crate::vertex::{
    child::Child,
    pattern::Pattern,
};

#[cfg(test)]
mod tests;

#[allow(unused)]
pub fn pattern_from_widths(widths: impl IntoIterator<Item=usize>) -> Pattern {
    widths
        .into_iter()
        .enumerate()
        .map(|(i, w)| Child::new(i, w))
        .collect()
}
