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
use context_trace::{
    trace::has_graph::HasGraph,
    HashMap,
};
use tracing::{
    info,
    Level,
};
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

#[test]
fn prefix1() {
    //tracing_subscriber::fmt()
    //    .with_max_level(Level::DEBUG)
    //    .with_target(false)
    //    .init();
    let Env1 {
        graph,
        a,
        e,
        d,
        bc,
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

    let query = vec![*a, *bc, *d, *e];
    let res: IncompleteState =
        graph.find_ancestor(query).unwrap().try_into().unwrap();

    assert_eq!(res.start, *a);
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
            bottom_up: FromIterator::from_iter([]),
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
#[test]
fn postfix1() {
    //tracing_subscriber::fmt()
    //    .with_max_level(Level::DEBUG)
    //    .with_target(false)
    //    .init();
    let Env1 {
        graph,
        a,
        c,
        e,
        d,
        bc,
        abc,
        abcd,
        abc_d_id,
        a_bc_id,
        abcdef,
        abc_def_id,
        abcd_ef_id,
        ab_cdef_id,
        cdef,
        ghi,
        //def,
        ef,
        e_f_id,
        abcdefghi,
        abcd_efghi_id,
        abcdef_ghi_id,
        ..
    } = &*Env1::get_expected();

    let query = vec![*c, *d, *ef, *ghi];
    let res: IncompleteState =
        graph.find_ancestor(query).unwrap().try_into().unwrap();

    assert_eq!(res.start, *c);

    assert_eq!(
        res.cache.entries[&c.index],
        VertexCache {
            index: *c,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([]),
        },
    );

    assert_eq!(
        res.cache.entries[&abcdefghi.index],
        VertexCache {
            index: *abcdefghi,
            bottom_up: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*abcdef, 4),
                            SubLocation::new(*abcdef_ghi_id, 0)
                        )]),
                    },
                }
            )]),
            top_down: FromIterator::from_iter([]),
        },
    );
    assert_eq!(
        res.cache.entries[&abcdef.index],
        VertexCache {
            index: *abcdef,
            bottom_up: FromIterator::from_iter([(
                4.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*cdef, 4),
                            SubLocation::new(*ab_cdef_id, 1)
                        )]),
                    },
                }
            )]),
            top_down: FromIterator::from_iter([]),
        },
    );
    //assert_eq!(
    //    res.cache.entries[&ef.index],
    //    VertexCache {
    //        index: *ef,
    //        bottom_up: FromIterator::from_iter([]),
    //        top_down: FromIterator::from_iter([(
    //            2.into(),
    //            PositionCache {
    //                edges: Edges {
    //                    top: FromIterator::from_iter([]),
    //                    bottom: FromIterator::from_iter([]),
    //                },
    //            }
    //        )]),
    //    }
    //);
    assert_eq!(res.cache.entries.len(), 3);
}

#[test]
fn range1() {
    //tracing_subscriber::fmt()
    //    .with_max_level(Level::INFO)
    //    .with_target(false)
    //    .init();
    let Env1 {
        graph,
        a,
        c,
        e,
        d,
        bc,
        abc,
        abcd,
        abc_d_id,
        a_bc_id,
        abcdef,
        abc_def_id,
        abcd_ef_id,
        ab_cdef_id,
        cdef,
        ghi,
        bcd,
        bc_d_id,
        a_bcd_id,
        //def,
        ef,
        e_f_id,
        abcdefghi,
        abcd_efghi_id,
        abcdef_ghi_id,
        ..
    } = &*Env1::get_expected();

    let query = vec![*bc, *d, *e];
    let res: IncompleteState =
        graph.find_ancestor(query).unwrap().try_into().unwrap();

    assert_eq!(res.start, *bc);

    assert_eq!(
        res.cache.entries[&bc.index],
        VertexCache {
            index: *bc,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([]),
        },
    );
    //info!(
    //    "{:#?}",
    //    res.cache
    //        .entries
    //        .iter()
    //        .map(|(_, v)| graph.graph().index_string(v.index))
    //        .collect::<Vec<_>>()
    //);
    assert_eq!(
        res.cache.entries[&abcd.index],
        VertexCache {
            index: *abcd,
            bottom_up: FromIterator::from_iter([(
                3.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*bcd, 3),
                            SubLocation::new(*a_bcd_id, 1)
                        )]),
                    },
                }
            )]),
            top_down: FromIterator::from_iter([]),
        },
    );
    assert_eq!(
        res.cache.entries[&abcdef.index],
        VertexCache {
            index: *abcdef,
            bottom_up: FromIterator::from_iter([(
                3.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::up(*abcd, 3),
                            SubLocation::new(*abcd_ef_id, 0)
                        )]),
                    },
                }
            )]),
            top_down: FromIterator::from_iter([(
                3.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::down(*ef, 3),
                            SubLocation::new(*abcd_ef_id, 1)
                        )]),
                    },
                }
            )]),
        },
    );
    assert_eq!(
        res.cache.entries[&ef.index],
        VertexCache {
            index: *ef,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([(
                3.into(),
                PositionCache {
                    edges: Edges {
                        top: FromIterator::from_iter([]),
                        bottom: FromIterator::from_iter([(
                            DirectedKey::down(*e, 3),
                            SubLocation::new(*e_f_id, 0)
                        )]),
                    },
                }
            )]),
        }
    );
    assert_eq!(res.cache.entries.len(), 5);
}
