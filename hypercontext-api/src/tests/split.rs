use crate::{
    graph::vertex::{
        child::Child,
        pattern::pattern_width,
    },
    interval::side::{
        SplitBack,
        SplitFront,
        SplitSide,
    },
    tests::mock,
};
use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

#[test]
fn token_pos_split() {
    let pattern = mock::pattern_from_widths([1, 1, 3, 1, 1]);
    let width = pattern_width(&pattern);
    assert_eq!(
        SplitBack::token_pos_split(pattern.borrow() as &[Child], NonZeroUsize::new(2).unwrap(),),
        Some((2, None).into()),
    );
    assert_eq!(
        SplitFront::token_pos_split(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(width - 2).unwrap(),
        ),
        Some((2, None).into()),
    );
    assert_eq!(
        SplitFront::token_pos_split(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(width - 4).unwrap(),
        ),
        Some((2, NonZeroUsize::new(1)).into()),
    );
}
