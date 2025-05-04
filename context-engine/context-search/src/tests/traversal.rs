#![allow(unused)]
use std::iter::FromIterator;

use context_trace::{
    graph::vertex::{
        location::SubLocation,
        wide::Wide,
    },
    lab,
    tests::env::{
        Env1,
        TestEnv,
    },
    trace::cache::{
        key::directed::DirectedKey,
        position::{
            Edges,
            PositionCache,
        },
        vertex::VertexCache,
    },
};
use pretty_assertions::assert_eq;
use std::convert::TryInto;

use crate::{
    search::Searchable,
    traversal::result::IncompleteState,
};
use tracing::Level;
#[allow(unused)]
use {
    context_trace::{
        graph::vertex::{
            child::Child,
            pattern::pattern_width,
        },
        tests::mock,
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

pub fn build_traversal1() -> IncompleteState {
    let Env1 {
        graph, a, d, e, bc, ..
    } = &*Env1::get_expected();
    let query = vec![*a, *bc, *d, *e];
    graph.find_ancestor(query).unwrap().try_into().unwrap()
}

#[test]
fn traverse_graph1() {
    //tracing_subscriber::fmt()
    //    .with_max_level(Level::DEBUG)
    //    .with_target(false)
    //    .init();
    let res = build_traversal1();
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
    } = &*Env1::get_expected();

    assert_eq!(res.start, *a);
    //assert_eq!(res.end_state.width(), 5);
    //println!("{:#?}", res.cache.entries);

    assert_eq!(
        res.cache.entries[&a.index],
        VertexCache {
            index: *a,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([]),
        },
    );
    assert_eq!(
        res.cache.entries[&e.index],
        VertexCache {
            index: *e,
            top_down: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Default::default(),
                }
            )]),
            bottom_up: FromIterator::from_iter([]),
        },
    );
    //assert_eq!(
    //    res.cache.entries[&abc.index],
    //    VertexCache {
    //        index: *abc,
    //        bottom_up: FromIterator::from_iter([(
    //            1.into(),
    //            PositionCache {
    //                edges: Edges {
    //                    top: Default::default(),
    //                    bottom: FromIterator::from_iter([(
    //                        DirectedKey::up(*a, 0),
    //                        SubLocation::new(*a_bc_id, 0),
    //                    )]),
    //                },
    //            }
    //        )]),
    //        top_down: FromIterator::from_iter([]),
    //    }
    //);
    //assert_eq!(
    //    res.cache.entries[&abcd.index],
    //    VertexCache {
    //        index: *abcd,
    //        bottom_up: FromIterator::from_iter([(
    //            3.into(),
    //            PositionCache {
    //                edges: Edges {
    //                    top: Default::default(),
    //                    bottom: FromIterator::from_iter([(
    //                        DirectedKey::up(*abc, 1),
    //                        SubLocation::new(*abc_d_id, 0),
    //                    )]),
    //                },
    //            }
    //        )]),
    //        top_down: FromIterator::from_iter([]),
    //    }
    //);
    assert_eq!(
        res.cache.entries[&ef.index],
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
                }
            )]),
        }
    );

    assert_eq!(
        res.cache.entries[&abcdef.index],
        VertexCache {
            index: *abcdef,
            bottom_up: FromIterator::from_iter([
                //(
                //    4.into(),
                //    PositionCache {
                //        edges: Edges {
                //            top: FromIterator::from_iter([]),
                //            bottom: FromIterator::from_iter([(
                //                DirectedKey::up(*abcd, 3),
                //                SubLocation::new(*abcd_ef_id, 0),
                //            ),]),
                //        },
                //    }
                //)
            ]),
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
                }
            )]),
        },
    );
    assert_eq!(res.cache.entries.len(), 4);
}
