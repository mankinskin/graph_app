#![allow(unused)]
use std::iter::FromIterator;

use crate::{
    graph::vertex::{
        location::SubLocation,
        wide::Wide,
    },
    lab,
    search::Searchable,
    tests::env::{
        Env1,
        TestEnv,
    },
    traversal::{
        cache::{
            entry::{
                position::{
                    Edges,
                    PositionCache,
                },
                vertex::VertexCache,
            },
            key::directed::DirectedKey,
        },
        result::IncompleteState,
    },
};
use std::convert::TryInto;

#[allow(unused)]
use {
    super::mock,
    crate::{
        graph::vertex::{
            child::Child,
            pattern::pattern_width,
        },
        trace::child::{
            TraceBack,
            TraceFront,
            TraceSide,
        },
    },
    std::{
        borrow::Borrow,
        num::NonZeroUsize,
    },
};

#[test]
fn token_pos_split()
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

pub fn build_trace1() -> IncompleteState
{
    let Env1 {
        graph, a, d, e, bc, ..
    } = &Env1::build_expected();
    let query = vec![*a, *bc, *d, *e];
    graph.find_ancestor(query).unwrap().try_into().unwrap()
}

#[test]
fn trace_graph1()
{
    let res = build_trace1();
    let Env1 {
        a,
        e,
        abc,
        abcd,
        abc_d_id,
        a_bc_id,
        abcdef,
        //abc_def_id,
        abcd_ef_id,
        //def,
        ef,
        e_f_id,
        ..
    } = &Env1::build_expected();

    assert_eq!(res.start, *a);
    assert_eq!(res.end_state.width(), 5);

    assert_eq!(
        res.cache.entries[&lab!(a)],
        VertexCache {
            index: *a,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([]),
        },
    );
    assert_eq!(
        res.cache.entries[&lab!(abcd)],
        VertexCache {
            index: *abcd,
            bottom_up: FromIterator::from_iter([(
                3.into(),
                PositionCache {
                    edges: Edges {
                        top: Default::default(),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*abc, 1),
                            SubLocation::new(*abc_d_id, 0),
                        )]),
                    },
                    index: *abcd,
                    //waiting: Default::default(),
                }
            )]),
            top_down: FromIterator::from_iter([]),
        }
    );
    assert_eq!(
        res.cache.entries[&lab!(ef)],
        VertexCache {
            index: *ef,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::down(*e, 4),
                            SubLocation::new(*e_f_id, 0),
                        )]),
                    },
                    index: *ef,
                    //waiting: Default::default(),
                }
            )]),
        }
    );
    assert_eq!(
        res.cache.entries[&lab!(e)],
        VertexCache {
            index: *e,
            top_down: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Default::default(),
                    index: *e,
                    //waiting: Default::default(),
                }
            )]),
            bottom_up: FromIterator::from_iter([]),
        },
    );
    assert_eq!(
        res.cache.entries[&lab!(abc)],
        VertexCache {
            index: *abc,
            bottom_up: FromIterator::from_iter([(
                1.into(),
                PositionCache {
                    edges: Edges {
                        top: Default::default(),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*a, 0),
                            SubLocation::new(*a_bc_id, 0),
                        )]),
                    },
                    index: *abc,
                    //waiting: Default::default(),
                }
            )]),
            top_down: FromIterator::from_iter([]),
        }
    );
    assert_eq!(
        res.cache.entries[&lab!(abcdef)],
        VertexCache {
            index: *abcdef,
            bottom_up: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*abcd, 3),
                            SubLocation::new(*abcd_ef_id, 0),
                        ),]),
                    },
                    index: *abcdef,
                    //waiting: Default::default(),
                }
            )]),
            top_down: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::down(*ef, 4),
                            SubLocation::new(*abcd_ef_id, 1),
                        )]),
                    },
                    index: *abcdef,
                    //waiting: Default::default(),
                }
            )]),
        },
    );
    assert_eq!(res.cache.entries.len(), 6);
}
