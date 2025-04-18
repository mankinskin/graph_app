#![allow(unused)]

use std::iter::FromIterator;

use crate::search::Searchable;
use itertools::*;
use pretty_assertions::{
    assert_eq,
    assert_matches,
};

#[cfg(test)]
use context_trace::tests::env::Env1;

use crate::traversal::{
    result::{
        FinishedKind,
        FinishedState,
    },
    state::{
        cursor::PatternRangeCursor,
        end::{
            EndKind,
            EndReason,
            EndState,
            PostfixEnd,
        },
    },
};
use context_trace::{
    graph::{
        getters::ErrorReason,
        kind::BaseGraphKind,
        vertex::{
            child::Child,
            location::{
                child::ChildLocation,
                pattern::PatternLocation,
                SubLocation,
            },
            token::Token,
        },
        Hypergraph,
        HypergraphRef,
    },
    lab,
    path::structs::{
        role_path::RolePath,
        rooted::{
            role_path::RootedRolePath,
            root::IndexRoot,
            RootedRangePath,
        },
        sub_path::SubPath,
    },
    tests::env::TestEnv,
    trace::{
        cache::{
            key::directed::DirectedKey,
            position::{
                Edges,
                PositionCache,
            },
            vertex::VertexCache,
            TraceCache,
        },
        has_graph::HasGraph,
    },
    HashMap,
    HashSet,
};

#[test]
fn find_sequence() {
    let Env1 {
        graph,
        abc,
        ababababcdefghi,
        a,
        ..
    } = &Env1::build_expected();
    assert_eq!(
        graph.find_sequence("a".chars()),
        Err(ErrorReason::SingleIndex(*a)),
    );
    let query = graph.graph().expect_token_children("abc".chars());
    let abc_found = graph.find_ancestor(&query);
    assert_eq!(
        abc_found.map(|r| r.kind),
        Ok(FinishedKind::Complete(*abc)),
        "abc"
    );
    let query = graph
        .graph()
        .expect_token_children("ababababcdefghi".chars());
    let ababababcdefghi_found = graph.find_ancestor(&query);
    assert_eq!(
        ababababcdefghi_found.map(|r| r.kind),
        Ok(FinishedKind::Complete(*ababababcdefghi)),
        "ababababcdefghi"
    );
}

#[test]
fn find_parent1() {
    let Env1 {
        graph,
        a,
        b,
        c,
        d,
        ab,
        bc,
        abc,
        ..
    } = &Env1::build_expected();
    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    let a_bc_d_pattern =
        vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    let bc_pattern = vec![Child::new(bc, 2)];
    let a_b_c_pattern =
        vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

    let query = bc_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Err(ErrorReason::SingleIndex(*bc)),
        "bc"
    );
    let query = b_c_pattern;
    assert_matches!(
        graph.find_parent(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *bc,
        "b_c"
    );
    let query = ab_c_pattern;
    assert_matches!(
        graph.find_parent(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
        "ab_c"
    );
    // enable when bfs for parent-child batches is implemented
    //let query = a_bc_pattern;
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_bc"
    //);
    //let query = a_bc_d_pattern;
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_bc_d"
    //);
    //let query = a_b_c_pattern.clone();
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_b_c"
    //);
    //let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_b_c_c"
    //);
}

#[test]
fn find_ancestor1() {
    let Env1 {
        graph,
        a,
        b,
        c,
        d,
        e,
        f,
        g,
        h,
        i,
        ab,
        bc,
        abc,
        abcd,
        ababababcdefghi,
        ..
    } = Env1::build_expected();
    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    let a_bc_d_pattern =
        vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    let bc_pattern = vec![Child::new(bc, 2)];
    let a_b_c_pattern =
        vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

    let query = bc_pattern;
    assert_eq!(
        graph.find_ancestor(query),
        Err(ErrorReason::SingleIndex(bc)),
        "bc"
    );

    let query = b_c_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == bc,
        "b_c"
    );

    println!("################## A_BC");
    let query = a_bc_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == abc,
        "a_bc"
    );

    let query = ab_c_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == abc,
        "ab_c"
    );

    let query = a_bc_d_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == abcd,
        "a_bc_d"
    );

    let query = a_b_c_pattern.clone();
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == abc,
        "a_b_c"
    );

    let query = vec![a, b, a, b, a, b, a, b, c, d, e, f, g, h, i];
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == ababababcdefghi,
        "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
    );

    let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == abc,
        "a_b_c_c"
    );
}

#[test]
fn find_ancestor2() {
    let mut graph = Hypergraph::<BaseGraphKind>::default();
    let (a, b, _w, x, y, z) = graph
        .insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])
        .into_iter()
        .next_tuple()
        .unwrap();
    let ab = graph.insert_pattern(vec![a, b]); // 6
    let by = graph.insert_pattern(vec![b, y]); // 7
    let yz = graph.insert_pattern(vec![y, z]); // 8
    let xa = graph.insert_pattern(vec![x, a]); // 9
    let xab = graph.insert_patterns([vec![x, ab], vec![xa, b]]); // 10
    let (xaby, xaby_ids) =
        graph.insert_patterns_with_ids([vec![xab, y], vec![xa, by]]); // 11
    let xa_by_id = xaby_ids[1];
    //assert_eq!(xa_by_id, 7);
    let (xabyz, xabyz_ids) =
        graph.insert_patterns_with_ids([vec![xaby, z], vec![xab, yz]]); // 12
    let xaby_z_id = xabyz_ids[0];
    //assert_eq!(xaby_z_id, 8);
    let graph = HypergraphRef::from(graph);
    let query = vec![by, z];
    let byz_found = graph.find_ancestor(&query);
    assert_eq!(
        byz_found,
        Ok(FinishedState {
            kind: FinishedKind::Incomplete(EndState {
                root_pos: 2.into(),
                reason: EndReason::QueryEnd,
                //target: DirectedKey::down(z, 2),
                kind: EndKind::Postfix(PostfixEnd {
                    inner_width: 1,
                    path: RootedRolePath {
                        root: IndexRoot {
                            location: PatternLocation {
                                parent: xabyz,
                                id: xaby_z_id,
                            },
                        },
                        role_path: RolePath {
                            sub_path: SubPath {
                                root_entry: 0,
                                path: vec![ChildLocation {
                                    parent: xaby,
                                    pattern_id: xa_by_id,
                                    sub_index: 1,
                                },],
                            },
                            _ty: Default::default(),
                        },
                    },
                }),
                cursor: PatternRangeCursor {
                    path: RootedRangePath {
                        root: query.clone(),
                        start: RolePath {
                            sub_path: SubPath {
                                root_entry: 0,
                                path: vec![],
                            },
                            _ty: Default::default(),
                        },
                        end: RolePath {
                            sub_path: SubPath {
                                root_entry: 1,
                                path: vec![],
                            },
                            _ty: Default::default(),
                        },
                    },
                    relative_pos: 3.into(),
                },
            }),
            root: xabyz,
            start: by,
            cache: TraceCache {
                entries: HashMap::from_iter([
                    (
                        xabyz.index,
                        VertexCache {
                            index: xabyz,
                            top_down: FromIterator::from_iter([
                                    //(2.into(), PositionCache {
                                    //    edges: Edges {
                                    //        bottom: HashMap::from_iter([
                                    //            //(
                                    //            //    DirectedKey::down(z, 2),
                                    //            //    SubLocation::new(8, 1),
                                    //            //),
                                    //        ]),
                                    //        top: Default::default(),
                                    //    },
                                    //    index: xabyz,
                                    //    waiting: Default::default(),
                                    //})
                                ]),
                            bottom_up: FromIterator::from_iter([(
                                2.into(),
                                PositionCache {
                                    edges: Edges {
                                        bottom: HashMap::from_iter([(
                                            DirectedKey::up(xaby, 2),
                                            SubLocation::new(xaby_z_id, 0),
                                        ),]),
                                        top: Default::default(),
                                    },
                                    index: xabyz,
                                    //waiting: Default::default(),
                                }
                            )]),
                        }
                    ),
                    (
                        xaby.index,
                        VertexCache {
                            index: xaby,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([(
                                2.into(),
                                PositionCache {
                                    edges: Edges {
                                        bottom: HashMap::from_iter([(
                                            DirectedKey::up(by, 0),
                                            SubLocation::new(xa_by_id, 1)
                                        )]),
                                        top: HashSet::from_iter([
                                                //DirectedKey {
                                                //    index: xabyz,
                                                //    pos: 2.into(),
                                                //},
                                            ]),
                                    },
                                    index: xaby,
                                    //waiting: Default::default(),
                                }
                            )]),
                        }
                    ),
                    (
                        by.index,
                        VertexCache {
                            index: by,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([
                                    //(2.into(), PositionCache {
                                    //    edges: Edges {
                                    //        bottom: Default::default(),
                                    //        top: HashSet::from_iter([
                                    //            //DirectedKey {
                                    //            //    index: xaby,
                                    //            //    pos: 2.into(),
                                    //            //},
                                    //        ]),
                                    //    },
                                    //    index: by,
                                    //    waiting: vec![],
                                    //})
                                ]),
                        }
                    ),
                    //(z.index, VertexCache {
                    //    index: z,
                    //    bottom_up: HashMap::from_iter([]),
                    //    top_down: HashMap::from_iter([
                    //        (2.into(), PositionCache {
                    //            edges: Edges {
                    //                top: HashSet::from_iter([
                    //                    //DirectedKey {
                    //                    //    index: xabyz,
                    //                    //    pos: 2.into(),
                    //                    //},
                    //                ]),
                    //                bottom: Default::default(),
                    //            },
                    //            index: z,
                    //            waiting: vec![],
                    //        })
                    //    ])
                    //}),
                ]),
            },
        }),
        "by_z"
    );
}

#[test]
fn find_ancestor3() {
    let mut graph = Hypergraph::<BaseGraphKind>::default();
    let (a, b, _w, x, y, z) = graph
        .insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])
        .into_iter()
        .next_tuple()
        .unwrap();
    // 6
    let ab = graph.insert_pattern(vec![a, b]);
    let by = graph.insert_pattern(vec![b, y]);
    let yz = graph.insert_pattern(vec![y, z]);
    let xa = graph.insert_pattern(vec![x, a]);
    // 10
    let (xab, xab_ids) =
        graph.insert_patterns_with_ids([vec![x, ab], vec![xa, b]]);
    let x_ab_id = xab_ids[0];
    //assert_eq!(x_ab_id, 4);
    // 11
    let (xaby, xaby_ids) =
        graph.insert_patterns_with_ids([vec![xab, y], vec![xa, by]]);
    let xab_y_id = xaby_ids[0];
    //assert_eq!(xab_y_id, 6);
    // 12
    let _xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
    let gr = HypergraphRef::from(graph);

    let query = vec![ab, y];
    let aby_found = gr.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(FinishedState {
            kind: FinishedKind::Incomplete(EndState {
                root_pos: 2.into(),
                reason: EndReason::QueryEnd,
                //target: y.top_down(2),
                kind: EndKind::Postfix(PostfixEnd {
                    inner_width: 1,
                    path: RootedRolePath {
                        root: IndexRoot {
                            location: PatternLocation {
                                parent: xaby,
                                id: xab_y_id,
                            },
                        },
                        role_path: RolePath {
                            sub_path: SubPath {
                                root_entry: 0,
                                path: vec![ChildLocation {
                                    parent: xab,
                                    pattern_id: x_ab_id,
                                    sub_index: 1,
                                },],
                            },
                            _ty: Default::default(),
                        },
                    },
                }),
                cursor: PatternRangeCursor {
                    path: RootedRangePath {
                        root: query.clone(),
                        start: RolePath {
                            sub_path: SubPath {
                                root_entry: 0,
                                path: vec![],
                            },
                            _ty: Default::default(),
                        },
                        end: RolePath {
                            sub_path: SubPath {
                                root_entry: 1,
                                path: vec![],
                            },
                            _ty: Default::default(),
                        },
                    },
                    relative_pos: 3.into(),
                },
            }),

            root: xaby,
            start: ab,
            cache: TraceCache {
                entries: HashMap::from_iter([
                    //(xabyz.index, VertexCache {
                    //    index: xabyz,
                    //    top_down: HashMap::from_iter([
                    //        (2.into(), PositionCache {
                    //            edges: Edges {
                    //                top: HashSet::default(),
                    //                bottom: HashMap::from_iter([
                    //                    (
                    //                        DirectedKey::down(yz, 2),
                    //                        SubLocation::new(9, 1),
                    //                    )
                    //                ]),
                    //            },
                    //            index: xabyz,
                    //            waiting: Default::default(),
                    //        })
                    //    ]),
                    //    bottom_up: HashMap::from_iter([
                    //        (2.into(), PositionCache {
                    //            edges: Edges {
                    //                top: HashSet::default(),
                    //                bottom: HashMap::from_iter([
                    //                    (
                    //                        DirectedKey::up(xab, 2),
                    //                        SubLocation::new(9, 0),
                    //                    )
                    //                ]),
                    //            },
                    //            index: xabyz,
                    //            waiting: Default::default(),
                    //        })
                    //    ])
                    //}),
                    (
                        xaby.index,
                        VertexCache {
                            index: xaby,
                            top_down: FromIterator::from_iter([
                                    //(2.into(), PositionCache {
                                    //    edges: Edges {
                                    //        bottom: HashMap::from_iter([
                                    //            (
                                    //                DirectedKey::down(y, 2),
                                    //                SubLocation::new(6, 1),
                                    //            ),
                                    //        ]),
                                    //        top: Default::default(),
                                    //    },
                                    //    index: xaby,
                                    //    waiting: Default::default(),
                                    //})
                                ]),
                            bottom_up: FromIterator::from_iter([(
                                2.into(),
                                PositionCache {
                                    edges: Edges {
                                        bottom: HashMap::from_iter([(
                                            DirectedKey::up(xab, 2),
                                            SubLocation::new(xab_y_id, 0),
                                        ),]),
                                        top: Default::default(),
                                    },
                                    index: xaby,
                                    //waiting: Default::default(),
                                }
                            )]),
                        }
                    ),
                    (
                        xab.index,
                        VertexCache {
                            index: xab,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([(
                                2.into(),
                                PositionCache {
                                    edges: Edges {
                                        bottom: HashMap::from_iter([(
                                            DirectedKey::up(ab, 0),
                                            SubLocation::new(x_ab_id, 1),
                                        )]),
                                        top: HashSet::from_iter([
                                                //DirectedKey {
                                                //    index: xabyz,
                                                //    pos: 2.into(),
                                                //},
                                                //DirectedKey {
                                                //    index: xaby,
                                                //    pos: 2.into(),
                                                //},
                                            ]),
                                    },
                                    index: xab,
                                    //waiting: vec![],
                                }
                            )]),
                        }
                    ),
                    (
                        ab.index,
                        VertexCache {
                            index: ab,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([
                                    //(2.into(), PositionCache {
                                    //    edges: Edges {
                                    //        bottom: HashMap::default(),
                                    //        top: HashSet::from_iter([
                                    //            //DirectedKey {
                                    //            //    index: xab,
                                    //            //    pos: 2.into(),
                                    //            //},
                                    //        ]),
                                    //    },
                                    //    index: ab,
                                    //    waiting: Default::default(),
                                    //})
                                ]),
                        }
                    ),
                    //(yz.index, VertexCache {
                    //    index: yz,
                    //    bottom_up: HashMap::from_iter([]),
                    //    top_down: HashMap::from_iter([
                    //        (2.into(), PositionCache {
                    //            edges: Edges {
                    //                top: HashSet::from_iter([
                    //                    //DirectedKey {
                    //                    //    index: xabyz,
                    //                    //    pos: 2.into(),
                    //                    //},
                    //                ]),
                    //                bottom: HashMap::from_iter([
                    //                    //(
                    //                    //    DirectedKey::up(y, 2),
                    //                    //    SubLocation::new(9, 0)
                    //                    //),
                    //                ]),
                    //            },
                    //            index: yz,
                    //            waiting: vec![],
                    //        })
                    //    ])
                    //}),
                    //(y.index, VertexCache {
                    //    index: y,
                    //    bottom_up: HashMap::from_iter([]),
                    //    top_down: HashMap::from_iter([
                    //        (2.into(), PositionCache {
                    //            edges: Edges {
                    //                top: HashSet::from_iter([
                    //                    //(DirectedKey {
                    //                    //    index: yz,
                    //                    //    pos: 2.into(),
                    //                    //}, SubLocation {
                    //                    //    pattern_id: 2,
                    //                    //    sub_index: 0,
                    //                    //}),
                    //                    //DirectedKey {
                    //                    //    index: xaby,
                    //                    //    pos: 2.into(),
                    //                    //},
                    //                ]),
                    //                bottom: Default::default(),
                    //            },
                    //            index: y,
                    //            waiting: vec![],
                    //        }),
                    //    ])
                    //}),
                ]),
            },
            //EndState {
            //    root_pos: 2.into(),
            //    //prev_pos: 2.into(),
            //    reason: EndReason::QueryEnd,
            //    matched: true,
            //    target: y.top_down(2),
            //    kind: EndKind::Range(
            //        RangeEnd {
            //            path: RootedRangePath {
            //                root: IndexRoot {
            //                    location: PatternLocation {
            //                        parent: xabyz,
            //                        pattern_id: 9,
            //                    },
            //                },
            //                start: RolePath {
            //                    sub_path: SubPath {
            //                        root_entry: 0,
            //                        path: vec![
            //                            ChildLocation {
            //                                parent: xab,
            //                                pattern_id: 4,
            //                                sub_index: 1,
            //                            },
            //                        ],
            //                    },
            //                    _ty: Default::default(),
            //                },
            //                end: RolePath {
            //                    sub_path: SubPath {
            //                        root_entry: 1,
            //                        path: vec![
            //                            ChildLocation {
            //                                parent: yz,
            //                                pattern_id: 2,
            //                                sub_index: 0,
            //                            },
            //                        ],
            //                    },
            //                    _ty: Default::default(),
            //                },
            //            },
            //        },
            //    ),
            //    query: RangeCursor {
            //        start: RolePath {
            //            sub_path: SubPath {
            //                root_entry: 0,
            //                path: vec![],
            //            },
            //            _ty: Default::default(),
            //        },
            //        end: RolePath {
            //            sub_path: SubPath {
            //                root_entry: 1,
            //                path: vec![],
            //            },
            //            _ty: Default::default(),
            //        },
            //        pos: 3.into(),
            //    },
            //},
        }),
        "ab_y"
    );
}
