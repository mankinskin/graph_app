#[allow(clippy::many_single_char_names)]

use super::*;
use crate::{
    graph::tests::context,
    Child,
    traversal::path::SearchPath,
};
use pretty_assertions::{
    assert_eq,
};
use itertools::*;


#[test]
fn find_parent1() {
    let Context {
        graph,
        a,
        b,
        c,
        d,
        ab,
        bc,
        abc,
        abcd,
        ..
     } = &*context();
    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    let bc_pattern = vec![Child::new(bc, 2)];
    let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

    let query = bc_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Err(NoMatch::SingleIndex(*bc)),
        "bc"
    );
    let query = b_c_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::new_complete(query, bc)),
        "b_c"
    );
    let query = a_bc_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::new_complete(query, abc)),
        "a_bc"
    );
    let query = ab_c_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::new_complete(query, abc)),
        "ab_c"
    );
    let query = a_bc_d_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::new_complete(query, abcd)),
        "a_bc_d"
    );
    let query = a_b_c_pattern.clone();
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::new_complete(query, abc)),
        "a_b_c"
    );
    let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult {
            path: FoundPath::Complete(*abc),
            query: QueryRangePath::new_range(
                query.clone(),
                0,
                query.len()-1
            ),
        }),
        "a_b_c_c"
    );
}

#[test]
fn find_ancestor1() {
    let Context {
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
     } = &*context();
    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    let bc_pattern = vec![Child::new(bc, 2)];
    let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

    let query = bc_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Err(NoMatch::SingleIndex(*bc)),
        "bc"
    );
    let query = b_c_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::new_complete(query, bc)),
        "b_c"
    );
    let query = a_bc_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::new_complete(query, abc)),
        "a_bc"
    );
    let query = ab_c_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::new_complete(query, abc)),
        "ab_c"
    );
    let query = a_bc_d_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::new_complete(query, abcd)),
        "a_bc_d"
    );
    let query = a_b_c_pattern.clone();
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::new_complete(query, abc)),
        "a_b_c"
    );
    let query =
        vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::new_complete(query, ababababcdefghi)),
        "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
    );
    let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult {
            path: FoundPath::Complete(*abc),
            query: QueryRangePath::new_range(
                query.clone(),
                0,
                query.len()-2
            ),
        }),
        "a_b_c_c"
    );
}

//#[test]
//fn find_ancestor2() {
//    let mut graph = Hypergraph::<BaseGraphKind>::default();
//    let (a, b, _w, x, y, z) = graph.insert_tokens([
//        Token::Element('a'),
//        Token::Element('b'),
//        Token::Element('w'),
//        Token::Element('x'),
//        Token::Element('y'),
//        Token::Element('z'),
//    ]).into_iter().next_tuple().unwrap();
//    let ab = graph.insert_pattern([a, b]);
//    let by = graph.insert_pattern([b, y]);
//    let yz = graph.insert_pattern([y, z]);
//    let xa = graph.insert_pattern([x, a]);
//    let xab = graph.insert_patterns([[x, ab], [xa, b]]);
//    let (xaby, xaby_ids) = graph.insert_patterns_with_ids([
//        vec![xab, y],
//        vec![xa, by],
//    ]);
//    let xa_by_id = xaby_ids[1];
//    let (xabyz, xabyz_ids) = graph.insert_patterns_with_ids([
//        vec![xaby, z],
//        vec![xab, yz],
//    ]);
//    let xaby_z_id = xabyz_ids[0];
//    let graph = HypergraphRef::from(graph);
//    let query = vec![by, z];
//    let byz_found = graph.find_ancestor(&query);
//    assert_eq!(
//        byz_found,
//        Ok(TraversalResult {
//            path: FoundPath::Path(
//                FoldResult {
//                    cache: TraversalCache {
//                        query_root: query.clone(),
//                        entries: HashMap::default()
//                    },
//                    final_states: vec![
//                    ]
//                }
//            //SearchPath {
//            //    start: RolePath {
//            //        path: vec![
//            //            xabyz.to_pattern_location(xaby_z_id)
//            //                .to_child_location(0),
//            //            ChildLocation {
//            //                parent: xaby,
//            //                pattern_id: xa_by_id,
//            //                sub_index: 1,
//            //            },
//            //        ],
//            //        width: 3,
//            //        child: by,
//            //        token_pos: 2,
//            //        _ty: Default::default(),
//            //    },
//            //    end: RolePath {
//            //        path: vec![
//            //            xabyz.to_pattern_location(xaby_z_id)
//            //                .to_child_location(1),
//            //        ],
//            //        width: 0,
//            //        child: z,
//            //        token_pos: 3,
//            //        _ty: Default::default(),
//            //    },
//            //}
//            ),
//            query: QueryRangePath::complete(query),
//        }),
//        "by_z"
//    );
//}

#[test]
fn find_ancestor3() {
    let mut graph = Hypergraph::<BaseGraphKind>::default();
    let (a, b, _w, x, y, z) = graph.insert_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ]).into_iter().next_tuple().unwrap();
    // 6
    let ab = graph.insert_pattern([a, b]);
    let by = graph.insert_pattern([b, y]);
    let yz = graph.insert_pattern([y, z]);
    let xa = graph.insert_pattern([x, a]);
    // 10
    let (xab, xab_ids) = graph.insert_patterns_with_ids([[x, ab], [xa, b]]);
    let x_ab_id = xab_ids[0];
    assert!(x_ab_id == 4);
    // 11
    let (xaby, xaby_ids) = graph.insert_patterns_with_ids([[xab, y], [xa, by]]);
    let xab_y_id = xaby_ids[0];
    assert!(xab_y_id == 6);
    // 12
    let _xabyz = graph.insert_patterns([[xaby, z], [xab, yz]]);

    let gr = HypergraphRef::from(graph);
    let query = vec![ab, y];
    let aby_found = gr.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(TraversalResult {
            path: FoundPath::Path(
                FoldResult {
                    cache: TraversalCache {
                        query_root: query.clone(),
                        entries: HashMap::from_iter([
                            (6, VertexCache {
                                positions: HashMap::from_iter([
                                    (2.into(), PositionCache {
                                        back_edges: Default::default(),
                                        index: Child {
                                            index: 6,
                                            width: 2,
                                        },
                                        waiting: Default::default(),
                                        num_parents: 0,
                                    })
                                ])
                            }),
                            (12, VertexCache {
                                positions: HashMap::from_iter([
                                    (2.into(), PositionCache {
                                        back_edges: HashMap::from_iter([
                                            (CacheKey {
                                                index: Child {
                                                    index: 10,
                                                    width: 3,
                                                },
                                                pos: 2.into(),
                                            }, CacheEdge::BottomUp(SubLocation {
                                                pattern_id: 9,
                                                sub_index: 0,
                                            })),
                                        ]),
                                        index: Child {
                                            index: 12,
                                            width: 5,
                                        },
                                        waiting: Default::default(),
                                        num_parents: 0,
                                    })
                                ])
                            }),
                            (4, VertexCache {
                                positions: HashMap::from_iter([
                                    (4.into(), PositionCache {
                                        back_edges: HashMap::from_iter([
                                            (CacheKey {
                                                index: Child {
                                                    index: 11,
                                                    width: 4,
                                                },
                                                pos: 2.into(),
                                            }, CacheEdge::TopDown(SubLocation {
                                                pattern_id: 6,
                                                sub_index: 1,
                                            })),
                                        ]),
                                        index: Child {
                                            index: 4,
                                            width: 1,
                                        },
                                        waiting: vec![
                                        ],
                                        num_parents: 1,
                                    })
                                ])
                            }),
                            (8, VertexCache {
                                positions: HashMap::from_iter([
                                    (4.into(), PositionCache {
                                        back_edges: HashMap::from_iter([
                                            (CacheKey {
                                                index: Child {
                                                    index: 12,
                                                    width: 5,
                                                },
                                                pos: 2.into(),
                                            }, CacheEdge::TopDown(SubLocation {
                                                pattern_id: 9,
                                                sub_index: 1,
                                            })),
                                        ]),
                                        index: Child {
                                            index: 8,
                                            width: 2,
                                        },
                                        waiting: vec![
                                        ],
                                        num_parents: 1,
                                    })
                                ])
                            }),
                            (11, VertexCache {
                                positions: HashMap::from_iter([
                                    (2.into(), PositionCache {
                                        back_edges: HashMap::from_iter([
                                            (CacheKey {
                                                index: Child {
                                                    index: 10,
                                                    width: 3,
                                                },
                                                pos: 2.into(),
                                            }, CacheEdge::BottomUp(SubLocation {
                                                pattern_id: 6,
                                                sub_index: 0,
                                            })),
                                        ]),
                                        index: Child {
                                            index: 11,
                                            width: 4,
                                        },
                                        waiting: Default::default(),
                                        num_parents: 0,
                                    })
                                ])
                            }),
                            (10, VertexCache {
                                positions: HashMap::from_iter([
                                    (2.into(), PositionCache {
                                        back_edges: HashMap::from_iter([
                                            (CacheKey {
                                                index: Child {
                                                    index: 6,
                                                    width: 2,
                                                },
                                                pos: 2.into(),
                                            }, CacheEdge::BottomUp(SubLocation {
                                                pattern_id: 4,
                                                sub_index: 1,
                                            })),
                                        ]),
                                        index: Child {
                                            index: 10,
                                            width: 3,
                                        },
                                        waiting: vec![
                                        ],
                                        num_parents: 0,
                                    })
                                ])
                            }),
                        ]),
                    },
                    final_states: vec![
                        FinalState {
                            num_parents: 0,
                            state: EndState {
                                root_pos: 2.into(),
                                kind: EndKind::Range(
                                    RangeEnd {
                                        kind: RangeKind::QueryEnd,
                                        path: RootedRangePath {
                                            root: IndexRoot {
                                                location: PatternLocation {
                                                    parent: Child {
                                                        index: 12,
                                                        width: 5,
                                                    },
                                                    pattern_id: 9,
                                                },
                                            },
                                            start: RolePath {
                                                sub_path: SubPath {
                                                    root_entry: 0,
                                                    path: vec![
                                                        ChildLocation {
                                                            parent: Child {
                                                                index: 10,
                                                                width: 3,
                                                            },
                                                            pattern_id: 4,
                                                            sub_index: 1,
                                                        },
                                                    ],
                                                },
                                                _ty: Default::default(),
                                            },
                                            end: RolePath {
                                                sub_path: SubPath {
                                                    root_entry: 1,
                                                    path: vec![
                                                        ChildLocation {
                                                            parent: Child {
                                                                index: 8,
                                                                width: 2,
                                                            },
                                                            pattern_id: 2,
                                                            sub_index: 0,
                                                        },
                                                    ],
                                                },
                                                _ty: Default::default(),
                                            },
                                        },
                                    },
                                ),
                                query: QueryState {
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
                                    pos: 4.into(),
                                },
                            },
                        },
                        FinalState {
                            num_parents: 0,
                            state: EndState {
                                root_pos: 2.into(),
                                kind: EndKind::Postfix(
                                    RootedRolePath {
                                        split_path: RootedSplitPath {
                                            root: IndexRoot {
                                                location: PatternLocation {
                                                    parent: Child {
                                                        index: 11,
                                                        width: 4,
                                                    },
                                                    pattern_id: 6,
                                                },
                                            },
                                            sub_path: SubPath {
                                                root_entry: 0,
                                                path: vec![
                                                    ChildLocation {
                                                        parent: Child {
                                                            index: 10,
                                                            width: 3,
                                                        },
                                                        pattern_id: 4,
                                                        sub_index: 1,
                                                    },
                                                ],
                                            },
                                        },
                                        _ty: Default::default(),
                                    },
                                ),
                                query: QueryState {
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
                                    pos: 4.into(),
                                },
                            },
                        },
                    ]
                }
            //SearchPath {
            //    start: RolePath {
            //        path: vec![
            //            xaby.to_pattern_location(xab_y_id)
            //                .to_child_location(0),
            //            ChildLocation {
            //                parent: xab,
            //                pattern_id: x_ab_id,
            //                sub_index: 1,
            //            },
            //        ],
            //        child: ab,
            //        width: 3,
            //        token_pos: 1,
            //        _ty: Default::default(),
            //    },
            //    end: RolePath {
            //        path: vec![
            //            xaby.to_pattern_location(xab_y_id)
            //                .to_child_location(1),
            //        ],
            //        width: 0,
            //        child: y,
            //        token_pos: 3,
            //        _ty: Default::default(),
            //    },
            //}
            ),
            query: QueryRangePath::complete(query),
        }),
        "ab_y"
    );
}

#[test]
fn find_sequence() {
    let Context {
        graph,
        abc,
        ababababcdefghi,
        a,
        ..
     } = &*context();
    assert_eq!(
        graph.find_sequence("a".chars()),
        Err(NoMatch::SingleIndex(*a)),
    );
    let query = graph.graph().expect_token_pattern("abc".chars());
    let abc_found = graph.find_ancestor(&query);
    assert_eq!(
        abc_found,
        Ok(TraversalResult::new_complete(query, abc)),
        "abc"
    );
    let query = graph.graph().expect_token_pattern("ababababcdefghi".chars());
    let ababababcdefghi_found = graph.find_ancestor(&query);
    assert_eq!(
        ababababcdefghi_found,
        Ok(TraversalResult::new_complete(query, ababababcdefghi)),
        "ababababcdefghi"
    );
}