#![allow(unused)]
use std::iter::FromIterator;

use context_trace::*;
use pretty_assertions::{
    assert_eq,
    assert_matches,
};
use std::convert::TryInto;

use crate::{
    fold::result::IncompleteState,
    search::Searchable,
    traversal::state::{
        cursor::PathCursor,
        end::{
            EndKind,
            EndReason,
            EndState,
        },
    },
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

    assert_eq!(
        res.clone(),
        IncompleteState {
            start: *a,
            root: IndexWithPath::new(*abcdef, res.root.path),
            end_state: EndState {
                cursor: PathCursor {
                    relative_pos: 5.into(),
                    path: res.end_state.cursor.path,
                },
                kind: res.end_state.kind,
                reason: EndReason::QueryEnd
            },
            cache: TraceCache {
                entries: HashMap::from_iter([
                    (
                        a.index,
                        VertexCache {
                            index: *a,
                            bottom_up: FromIterator::from_iter([]),
                            top_down: FromIterator::from_iter([]),
                        },
                    ),
                    (
                        e.index,
                        VertexCache {
                            index: *e,
                            top_down: FromIterator::from_iter([(
                                4.into(),
                                PositionCache::default(),
                            )]),
                            bottom_up: FromIterator::from_iter([]),
                        },
                    ),
                    (
                        ef.index,
                        VertexCache {
                            index: *ef,
                            bottom_up: FromIterator::from_iter([]),
                            top_down: FromIterator::from_iter([(
                                4.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::down(*e, 4),
                                        SubLocation::new(*e_f_id, 0),
                                    )]),
                                }
                            )]),
                        }
                    ),
                    (
                        abcdef.index,
                        VertexCache {
                            index: *abcdef,
                            bottom_up: FromIterator::from_iter([]),
                            top_down: FromIterator::from_iter([(
                                4.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::down(*ef, 4),
                                        SubLocation::new(*abcd_ef_id, 1),
                                    )]),
                                }
                            )]),
                        },
                    ),
                ]),
            },
        }
    );
}
#[test]
fn postfix1() {
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

    assert_eq!(
        res.clone(),
        IncompleteState {
            start: *c,
            root: IndexWithPath::new(*abcdefghi, res.root.path),
            end_state: EndState {
                cursor: PathCursor {
                    relative_pos: 7.into(),
                    path: res.end_state.cursor.path,
                },
                kind: res.end_state.kind,
                reason: EndReason::QueryEnd
            },
            cache: TraceCache {
                entries: HashMap::from_iter([
                    (
                        c.index,
                        VertexCache {
                            index: *c,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([]),
                        },
                    ),
                    (
                        abcdef.index,
                        VertexCache {
                            index: *abcdef,
                            bottom_up: FromIterator::from_iter([(
                                4.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::up(*cdef, 4),
                                        SubLocation::new(*ab_cdef_id, 1)
                                    )]),
                                }
                            )]),
                            top_down: FromIterator::from_iter([]),
                        },
                    ),
                    (
                        abcdefghi.index,
                        VertexCache {
                            index: *abcdefghi,
                            bottom_up: FromIterator::from_iter([(
                                4.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::up(*abcdef, 4),
                                        SubLocation::new(*abcdef_ghi_id, 0)
                                    )]),
                                }
                            )]),
                            top_down: FromIterator::from_iter([]),
                        },
                    ),
                ]),
            },
        }
    );
}

#[test]
fn range1() {
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

    assert_eq!(
        res.clone(),
        IncompleteState {
            start: *bc,
            root: IndexWithPath::new(*abcdef, res.root.path),
            end_state: EndState {
                cursor: PathCursor {
                    relative_pos: 4.into(),
                    path: res.end_state.cursor.path,
                },
                kind: res.end_state.kind,
                reason: EndReason::QueryEnd
            },
            cache: TraceCache {
                entries: HashMap::from_iter([
                    (
                        bc.index,
                        VertexCache {
                            index: *bc,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([]),
                        },
                    ),
                    (
                        abcdef.index,
                        VertexCache {
                            index: *abcdef,
                            bottom_up: FromIterator::from_iter([(
                                3.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::up(*abcd, 3),
                                        SubLocation::new(*abcd_ef_id, 0)
                                    )]),
                                }
                            )]),
                            top_down: FromIterator::from_iter([(
                                3.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::down(*ef, 3),
                                        SubLocation::new(*abcd_ef_id, 1)
                                    )]),
                                }
                            )]),
                        },
                    ),
                    (
                        abcd.index,
                        VertexCache {
                            index: *abcd,
                            bottom_up: FromIterator::from_iter([(
                                3.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::up(*bcd, 3),
                                        SubLocation::new(*a_bcd_id, 1)
                                    )]),
                                }
                            )]),
                            top_down: FromIterator::from_iter([]),
                        },
                    ),
                    (
                        ef.index,
                        VertexCache {
                            index: *ef,
                            bottom_up: FromIterator::from_iter([]),
                            top_down: FromIterator::from_iter([(
                                3.into(),
                                PositionCache {
                                    top: FromIterator::from_iter([]),
                                    bottom: FromIterator::from_iter([(
                                        DirectedKey::down(*e, 3),
                                        SubLocation::new(*e_f_id, 0)
                                    )]),
                                }
                            )]),
                        }
                    ),
                    (
                        e.index,
                        VertexCache {
                            index: *e,
                            top_down: FromIterator::from_iter([(
                                3.into(),
                                PositionCache::default(),
                            )]),
                            bottom_up: FromIterator::from_iter([]),
                        },
                    ),
                ]),
            },
        }
    );
}
