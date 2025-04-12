use crate::{
    graph::vertex::{
        child::Child,
        pattern::pattern_width,
    },
    interval::side::{
        TraceBack,
        TraceFront,
        TraceSide,
    },
    tests::mock,
};
use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

#[test]
fn trace_child_pos()
{
    let pattern = mock::pattern_from_widths([1, 1, 3, 1, 1]);
    let width = pattern_width(&pattern);
    assert_eq!(
        TraceBack::trace_child_pos(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(2).unwrap(),
        ),
        Some((2, None).into()),
    );
    assert_eq!(
        TraceFront::trace_child_pos(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(width - 2).unwrap(),
        ),
        Some((2, None).into()),
    );
    assert_eq!(
        TraceFront::trace_child_pos(
            pattern.borrow() as &[Child],
            NonZeroUsize::new(width - 4).unwrap(),
        ),
        Some((2, NonZeroUsize::new(1)).into()),
    );
}
