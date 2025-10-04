use std::{
    collections::{
        BTreeMap,
        VecDeque,
    },
    iter::FromIterator,
};

use pretty_assertions::assert_eq;

use crate::{
    build_split_cache,
    build_trace_cache,
    insert_patterns,
    interval::{
        IntervalGraph,
        init::InitInterval,
    },
    nz,
    split::{
        cache::{
            SplitCache,
            position::{
                PosKey,
                SplitPositionCache,
            },
            vertex::SplitVertexCache,
        },
        trace::states::SplitStates,
        vertex::output::RootMode,
    },
};
use context_search::*;
use context_trace::*;
fn build_split_cache1(env: &Env1) -> SplitCache {
    let Env1 {
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
    } = env;
    build_split_cache!(
        RootMode::Prefix,
        ef => {
            { abcdef: 5, def: 2, cdef: 3 } -> 1 => {
                e_f_id => (1, None)
            }
        },
        def => {
            { abcdef: 5, cdef: 3 } -> 2 => {
                d_ef_id => (1, Some(nz!(1)))
            }
        },
        cdef => {
            { abcdef: 5 } -> 3 => {
                c_def_id => (1, Some(nz!(2))),
                cd_ef_id => (1, Some(nz!(1)))
            }
        },
        abcdef => {
            {} -> 5 => {
                abcd_ef_id => (1, Some(nz!(1))),
                abc_def_id => (1, Some(nz!(2))),
                ab_cdef_id => (1, Some(nz!(3))),
            }
        }
    )
}
#[test]
fn test_split_cache1() {
    let env @ Env1 {
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
    } = &*Env1::get_expected_mut();
    assert_eq!(build_split_cache1(env), SplitCache {
        root_mode: RootMode::Prefix,
        entries: HashMap::from_iter([
            (ef.index, SplitVertexCache {
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
                            PosKey {
                                index: *cdef,
                                pos: nz!(3),
                            },
                        ]),
                        pattern_splits: HashMap::from_iter([(
                            *e_f_id,
                            ChildTracePos {
                                inner_offset: None,
                                sub_index: 1,
                            }
                        )])
                    }
                )])
            }),
            (def.index, SplitVertexCache {
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
                            ChildTracePos {
                                inner_offset: Some(nz!(1)),
                                sub_index: 1,
                            }
                        )])
                    }
                )])
            }),
            (cdef.index, SplitVertexCache {
                positions: BTreeMap::from_iter([(
                    nz!(3),
                    SplitPositionCache {
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
                    }
                )])
            }),
            (abcdef.index, SplitVertexCache {
                positions: BTreeMap::from_iter([(
                    nz!(5),
                    SplitPositionCache {
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
                    }
                )])
            }),
        ])
    });
}

#[test]
fn interval_graph1() {
    let env = &mut *Env1::get_expected_mut();
    let graph = &mut env.graph;
    let Env1 {
        a,
        d,
        e,
        bc,
        abcdef,
        ef,
        ..
    } = env;
    let query = vec![*a, *bc, *d, *e];
    let res: IncompleteState =
        graph.find_ancestor(query).unwrap().try_into().unwrap();
    let init = InitInterval::from(res);
    let interval = IntervalGraph::from((&mut *graph, init));
    assert_eq!(interval.clone(), IntervalGraph {
        root: *abcdef,
        states: SplitStates {
            leaves: vec![PosKey::new(*ef, 1)].into(),
            queue: VecDeque::default(),
        },
        cache: build_split_cache1(env)
    });
}

#[test]
fn interval_graph2() {
    init_tracing();
    let mut graph = HypergraphRef::default();
    insert_tokens!(graph, {a, b, c, d, e, f, g, h, i, j, k});
    insert_patterns!(graph,
        (cd, cd_id) => [c, d],
        (hi, hi_id) => [h, i],
        (efg, _efg_id) => [e, f, g],
        (cdefg, cdefg_id) => [cd, efg],
        (efghi, efghi_id) => [efg, hi],
    );
    insert_patterns!(graph,
        (cdefghi, cdefghi_ids) => [
            [cdefg, hi],
            [cd, efghi],
        ],
    );
    insert_patterns!(graph,
        (_abcdefghijk, _abcdefghijk_id) => [a, b, cdefghi, j, k],
    );
    let query = vec![d, e, f, g, h];
    let res: IncompleteState =
        graph.find_ancestor(query).unwrap().try_into().unwrap();
    let init = InitInterval::from(res);

    assert_eq!(init, InitInterval {
        root: cdefghi,
        end_bound: 5,
        cache: build_trace_cache!(
            d => (
                BU {},
                TD {}
            ),
            cd => (
                BU {
                    1 => d -> (cd_id, 1)
                },
                TD {}
            ),
            hi => (
                BU {},
                TD {
                    1 => h -> (hi_id, 0)
                }
            ),
            cdefg => (
                BU {
                    1 => cd -> (cdefg_id, 0)
                },
                TD {}
            ),
            h => (
                BU {},
                TD {}
            ),
            cdefghi => (
                BU {
                    4 => cdefg -> (cdefghi_ids[0], 0)
                },
                TD {
                    4 => hi -> (cdefghi_ids[0], 1)
                }
            ),
        )
    });
    let interval = IntervalGraph::from((&mut *graph.graph_mut(), init));
    assert_eq!(interval, IntervalGraph {
        root: cdefghi,
        states: SplitStates {
            leaves: vec![PosKey::new(cd, 1)].into(),
            queue: VecDeque::default(),
        },
        cache: build_split_cache!(
            RootMode::Infix,
            cd => {
                { cdefg: 2 } -> 1 => {
                    cd_id => (1, None)
                }
            },
            hi => {
                { efghi: 4 } -> 5 => {
                    hi_id => (1, None)
                }
            },
            cdefg => {
                { cdefghi: 1 } -> 1 => {
                    cdefg_id => (0, Some(nz!(1))),
                }
            },
            efghi => {
                { cdefghi: 2 } -> 4 => {
                    efghi_id => (1, Some(nz!(1))),
                }
            },
            cdefghi => {
                { } -> 1 => {
                    cdefghi_ids[1] => (0, Some(nz!(1))),
                },
                {} -> 6 => {
                    cdefghi_ids[0] => (1, Some(nz!(1))),
                },
            },
        )
    });
}
