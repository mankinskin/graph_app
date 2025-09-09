#[cfg(test)]
use {
    crate::{
        fold::result::{
            FinishedKind,
            FinishedState,
        },
        search::Searchable,
        traversal::state::{
            cursor::PatternCursor,
            end::{
                postfix::PostfixEnd,
                EndKind,
                EndReason,
                EndState,
            },
        },
    },
    context_trace::{
        graph::{
            getters::{
                ErrorReason,
                IndexWithPath,
            },
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
        path::structs::{
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::IndexRoot,
            },
            sub_path::SubPath,
        },
        tests::env::{
            Env1,
            TestEnv,
        },
        trace::cache::{
            key::directed::DirectedKey,
            position::PositionCache,
            vertex::VertexCache,
            TraceCache,
        },
        HashMap,
        HashSet,
    },
    itertools::*,
    pretty_assertions::{
        assert_eq,
        assert_matches,
    },
};

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
    } = &*Env1::get_expected();
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
        graph.find_ancestor(&query),
        Err(ErrorReason::SingleIndex(Box::new(IndexWithPath {
            index: *bc,
            path: query.into()
        }))),
        "bc"
    );

    let query = b_c_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *bc,
        "b_c"
    );

    println!("################## A_BC");
    let query = a_bc_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
        "a_bc"
    );

    let query = ab_c_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
        "ab_c"
    );

    let query = a_bc_d_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abcd,
        "a_bc_d"
    );

    let query = a_b_c_pattern.clone();
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
        "a_b_c"
    );

    let query: Vec<_> = [a, b, a, b, a, b, a, b, c, d, e, f, g, h, i]
        .into_iter()
        .cloned()
        .collect();
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *ababababcdefghi,
        "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
    );

    let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
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
    let byz_found = graph.find_ancestor(&query).unwrap();

    assert_eq!(
        byz_found.clone(),
        FinishedState {
            start: by,
            root: IndexWithPath {
                index: xabyz,
                path: byz_found.root.path.clone(),
            },
            kind: FinishedKind::Incomplete(Box::new(EndState {
                reason: EndReason::QueryEnd,
                //target: DirectedKey::down(z, 2),
                kind: EndKind::Postfix(PostfixEnd {
                    //inner_width: 2,
                    root_pos: 2.into(),
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
                cursor: PatternCursor {
                    path: RootedRolePath {
                        root: query.clone(),
                        role_path: RolePath {
                            sub_path: SubPath {
                                root_entry: 1,
                                path: vec![],
                            },
                            _ty: Default::default(),
                        },
                    },
                    relative_pos: 3.into(),
                },
            })),
            cache: TraceCache {
                entries: HashMap::from_iter([
                    (
                        xabyz.index,
                        VertexCache {
                            index: xabyz,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([(
                                2.into(), // width of by
                                PositionCache {
                                    bottom: HashMap::from_iter([(
                                        DirectedKey::up(xaby, 2), // width of by
                                        SubLocation::new(xaby_z_id, 0),
                                    ),]),
                                    top: Default::default(),
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
                                2.into(), // width of by
                                PositionCache {
                                    bottom: HashMap::from_iter([(
                                        DirectedKey::up(by, 2), // width of by
                                        SubLocation::new(xa_by_id, 1)
                                    )]),
                                    top: HashSet::from_iter([]),
                                }
                            )]),
                        }
                    ),
                    (
                        by.index,
                        VertexCache {
                            index: by,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([]),
                        }
                    ),
                ]),
            }
        }
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
    let aby_found = gr.find_ancestor(&query).unwrap();
    assert_eq!(
        aby_found.clone(),
        FinishedState {
            start: ab,
            root: IndexWithPath {
                index: xaby,
                path: aby_found.root.path.clone(),
            },
            kind: FinishedKind::Incomplete(Box::new(EndState {
                reason: EndReason::QueryEnd,
                kind: EndKind::Postfix(PostfixEnd {
                    root_pos: 2.into(),
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
                cursor: PatternCursor {
                    path: RootedRolePath {
                        root: query.clone(),
                        role_path: RolePath {
                            sub_path: SubPath {
                                root_entry: 1,
                                path: vec![],
                            },
                            _ty: Default::default(),
                        },
                    },
                    relative_pos: 3.into(),
                },
            })),
            cache: TraceCache {
                entries: HashMap::from_iter([
                    (
                        xaby.index,
                        VertexCache {
                            index: xaby,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([(
                                2.into(),
                                PositionCache {
                                    bottom: HashMap::from_iter([(
                                        DirectedKey::up(xab, 2),
                                        SubLocation::new(xab_y_id, 0),
                                    ),]),
                                    top: Default::default(),
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
                                    bottom: HashMap::from_iter([(
                                        DirectedKey::up(ab, 2),
                                        SubLocation::new(x_ab_id, 1),
                                    )]),
                                    top: HashSet::from_iter([]),
                                }
                            )]),
                        }
                    ),
                    (
                        ab.index,
                        VertexCache {
                            index: ab,
                            top_down: FromIterator::from_iter([]),
                            bottom_up: FromIterator::from_iter([]),
                        }
                    ),
                ]),
            }
        }
    );
}
