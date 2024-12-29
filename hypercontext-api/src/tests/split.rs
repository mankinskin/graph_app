use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use crate::{
    split::side::{
        SplitBack,
        SplitFront,
        SplitSide,
    },
    mock,
};
use crate::graph::vertex::{
    child::Child,
    pattern::pattern_width,
};

#[test]
fn token_offset_split() {
    let pattern = mock::pattern_from_widths([1, 1, 3, 1, 1]);
    let width = pattern_width(&pattern);
    assert_eq!(
        SplitBack::token_offset_split(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(2).unwrap(),
        ),
        Some((2, None)),
    );
    assert_eq!(
        SplitFront::token_offset_split(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(width - 2).unwrap(),
        ),
        Some((2, None)),
    );
    assert_eq!(
        SplitFront::token_offset_split(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(width - 4).unwrap(),
        ),
        Some((2, NonZeroUsize::new(1))),
    );
}