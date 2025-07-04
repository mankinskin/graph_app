use std::{
    collections::BTreeMap,
    iter::FromIterator,
};

use pretty_assertions::assert_eq;

use crate::{
    interval::{
        IntervalGraph,
        init::InitInterval,
    },
    split::cache::{
        position::{
            PosKey,
            SplitPositionCache,
        },
        vertex::SplitVertexCache,
    },
};
use context_search::{
    fold::result::IncompleteState,
    search::Searchable,
};
use context_trace::{
    HashMap,
    HashSet,
    tests::env::{
        Env1,
        TestEnv,
    },
    trace::child::ChildTracePos,
};
macro_rules! nz {
    ($x:expr) => {
        std::num::NonZeroUsize::new($x).unwrap()
    };
}
#[test]
fn interval_graph1() {
    let Env1 {
        graph,
        a,
        d,
        e,
        bc,
        def,
        d_ef_id,
        c_def_id,
        cd_ef_id,
        cdef,
        abcdef,
        abcd_ef_id,
        ab_cdef_id,
        abc_def_id,
        ef,
        e_f_id,
        ..
    } = &mut *Env1::get_expected_mut();
    let query = vec![*a, *bc, *d, *e];
    let res: IncompleteState =
        graph.find_ancestor(query).unwrap().try_into().unwrap();
    let init = InitInterval::from(res);
    let interval = IntervalGraph::from((&mut *graph, init));
    assert_eq!(interval.cache[&ef.index], SplitVertexCache {
        positions: BTreeMap::from_iter([(nz!(1), SplitPositionCache {
            top: HashSet::from_iter([
                PosKey {
                    index: *abcdef,
                    pos: nz!(5),
                },
                PosKey {
                    index: *def,
                    pos: nz!(2),
                },
                PosKey {
                    index: *cdef,
                    pos: nz!(3),
                },
            ]),
            pattern_splits: HashMap::from_iter([(*e_f_id, ChildTracePos {
                inner_offset: None,
                sub_index: 1,
            })])
        })])
    },);
    assert_eq!(interval.cache[&def.index], SplitVertexCache {
        positions: BTreeMap::from_iter([(nz!(2), SplitPositionCache {
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
            pattern_splits: HashMap::from_iter([(*d_ef_id, ChildTracePos {
                inner_offset: Some(nz!(1)),
                sub_index: 1,
            })])
        })])
    },);
    assert_eq!(interval.cache[&cdef.index], SplitVertexCache {
        positions: BTreeMap::from_iter([(nz!(3), SplitPositionCache {
            top: HashSet::from_iter([PosKey {
                index: *abcdef,
                pos: nz!(5),
            },]),
            pattern_splits: HashMap::from_iter([
                (*c_def_id, ChildTracePos {
                    inner_offset: Some(nz!(2)),
                    sub_index: 1,
                },),
                (*cd_ef_id, ChildTracePos {
                    inner_offset: Some(nz!(1)),
                    sub_index: 1,
                },)
            ])
        })])
    },);
    assert_eq!(interval.cache[&abcdef.index], SplitVertexCache {
        positions: BTreeMap::from_iter([(nz!(5), SplitPositionCache {
            top: HashSet::from_iter([]),
            pattern_splits: HashMap::from_iter([
                (*abcd_ef_id, ChildTracePos {
                    inner_offset: Some(nz!(1)),
                    sub_index: 1,
                }),
                (*abc_def_id, ChildTracePos {
                    inner_offset: Some(nz!(2)),
                    sub_index: 1,
                }),
                (*ab_cdef_id, ChildTracePos {
                    inner_offset: Some(nz!(3)),
                    sub_index: 1,
                }),
            ])
        })])
    },);
    assert_eq!(interval.cache.len(), 4);
}
