use std::{
    collections::BTreeMap,
    iter::FromIterator,
};

use pretty_assertions::assert_eq;

use crate::tests::trace::build_trace1;
use hypercontext_api::{
    interval::{
        cache::{
            position::SplitPositionCache,
            vertex::SplitVertexCache,
            PosKey,
        },
        InitInterval,
        IntervalGraph,
        PatternSplitPos,
    },
    lab,
    tests::env::{
        Env1,
        TestEnv,
    },
    HashMap,
    HashSet,
};

macro_rules! nz {
    ($x:expr) => {
        std::num::NonZeroUsize::new($x).unwrap()
    };
}
#[test]
fn interval_graph1() {
    let res = build_trace1();
    let Env1 {
        graph,
        def,
        d_ef_id,
        c_def_id,
        cdef,
        abcdef,
        abcd_ef_id,
        ab_cdef_id,
        abc_def_id,
        ef,
        e_f_id,
        ..
    } = &mut Env1::build_expected();
    let init = InitInterval::from(res);
    let interval = IntervalGraph::from((&mut *graph, init));
    assert_eq!(
        interval.vertices[&lab!(ef)],
        SplitVertexCache {
            positions: BTreeMap::from_iter([(
                nz!(1),
                SplitPositionCache {
                    top: HashSet::from_iter([
                        PosKey {
                            index: *abcdef,
                            pos: nz!(5),
                        },
                        PosKey {
                            index: *def,
                            pos: nz!(2),
                        },
                    ]),
                    pattern_splits: HashMap::from_iter([(
                        *e_f_id,
                        PatternSplitPos {
                            inner_offset: None,
                            sub_index: 1,
                        }
                    )])
                }
            )])
        },
    );
    assert_eq!(
        interval.vertices[&lab!(def)],
        SplitVertexCache {
            positions: BTreeMap::from_iter([(
                nz!(2),
                SplitPositionCache {
                    top: HashSet::from_iter([
                        PosKey {
                            index: *abcdef,
                            pos: nz!(5),
                        },
                        PosKey {
                            index: *cdef,
                            pos: nz!(3),
                        },
                    ]),
                    pattern_splits: HashMap::from_iter([(
                        *d_ef_id,
                        PatternSplitPos {
                            inner_offset: Some(nz!(1)),
                            sub_index: 1,
                        }
                    )])
                }
            )])
        },
    );
    assert_eq!(
        interval.vertices[&lab!(cdef)],
        SplitVertexCache {
            positions: BTreeMap::from_iter([(
                nz!(3),
                SplitPositionCache {
                    top: HashSet::from_iter([PosKey {
                        index: *abcdef,
                        pos: nz!(5),
                    },]),
                    pattern_splits: HashMap::from_iter([(
                        *c_def_id,
                        PatternSplitPos {
                            inner_offset: Some(nz!(2)),
                            sub_index: 1,
                        }
                    )])
                }
            )])
        },
    );
    assert_eq!(
        interval.vertices[&lab!(abcdef)],
        SplitVertexCache {
            positions: BTreeMap::from_iter([(
                nz!(5),
                SplitPositionCache {
                    top: HashSet::from_iter([]),
                    pattern_splits: HashMap::from_iter([
                        (
                            *abcd_ef_id,
                            PatternSplitPos {
                                inner_offset: Some(nz!(1)),
                                sub_index: 1,
                            }
                        ),
                        (
                            *abc_def_id,
                            PatternSplitPos {
                                inner_offset: Some(nz!(2)),
                                sub_index: 1,
                            }
                        ),
                        (
                            *ab_cdef_id,
                            PatternSplitPos {
                                inner_offset: Some(nz!(3)),
                                sub_index: 1,
                            }
                        ),
                    ])
                }
            )])
        },
    );
    assert_eq!(interval.vertices.len(), 4);
}
