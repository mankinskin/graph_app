use crate::*;

pub fn pattern_from_widths(widths: impl IntoIterator<Item=usize>) -> Pattern {
    widths.into_iter()
        .enumerate()
        .map(|(i, w)| Child::new(i, w))
        .collect()
}