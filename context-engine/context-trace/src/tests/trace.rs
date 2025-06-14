#![allow(unused)]
use crate::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
        pattern::pattern_width,
    },
    path::{
        RolePathUtils,
        accessors::{
            child::root::GraphRootChild,
            role::End,
        },
        mutators::move_path::key::TokenPosition,
    },
    tests::mock,
    trace::{
        cache::key::{
            directed::{
                DirectedKey,
                up::UpKey,
            },
            props::{
                CursorPosition,
                RootKey,
                TargetKey,
            },
        },
        child::{
            TraceBack,
            TraceFront,
            TraceSide,
        },
    },
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
