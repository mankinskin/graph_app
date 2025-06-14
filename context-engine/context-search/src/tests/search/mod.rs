#![allow(unused)]

pub mod ancestor;
pub mod consecutive;
pub mod parent;

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
        cursor::{
            PatternPrefixCursor,
            PatternRangeCursor,
        },
        end::{
            postfix::PostfixEnd,
            range::RangeEnd,
            EndKind,
            EndReason,
            EndState,
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
            key::directed::{
                down::DownKey,
                DirectedKey,
            },
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
use tracing::{
    info,
    Level,
};

#[test]
fn find_sequence() {
    let Env1 {
        graph,
        abc,
        ababababcdefghi,
        a,
        ..
    } = &*Env1::get_expected();
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
fn find_pattern1() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .init();
    let mut graph =
        context_trace::graph::Hypergraph::<BaseGraphKind>::default();
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
    // index 6
    let (yz, y_z_id) = graph.insert_pattern_with_id(vec![y, z]);
    let (xab, x_a_b_id) = graph.insert_pattern_with_id(vec![x, a, b]);
    let _xyz = graph.insert_pattern(vec![x, yz]);
    let _xabz = graph.insert_pattern(vec![xab, z]);
    let (xabyz, xab_yz_id) = graph.insert_pattern_with_id(vec![xab, yz]);

    let graph_ref = HypergraphRef::from(graph);

    let query = vec![a, b, y, x];
    let aby_found = graph_ref
        .find_ancestor(query.clone())
        .expect("Search failed");
    //info!("{:#?}", aby);

    assert_eq!(
        aby_found.cache.entries[&xab.index],
        VertexCache {
            index: xab,
            bottom_up: FromIterator::from_iter([(
                1.into(),
                PositionCache {
                    edges: Edges {
                        bottom: HashMap::from_iter([(
                            DirectedKey::up(a, 1),
                            SubLocation::new(x_a_b_id.unwrap(), 1)
                        )]),
                        top: HashSet::from_iter([]),
                    },
                }
            )]),
            top_down: FromIterator::from_iter([]),
        }
    );
    assert_eq!(
        aby_found.cache.entries[&xabyz.index],
        VertexCache {
            index: xabyz,
            bottom_up: FromIterator::from_iter([(
                2.into(),
                PositionCache {
                    edges: Edges {
                        bottom: HashMap::from_iter([(
                            DirectedKey::up(xab, 1),
                            SubLocation::new(xab_yz_id.unwrap(), 0)
                        )]),
                        top: HashSet::from_iter([]),
                    },
                }
            )]),
            top_down: FromIterator::from_iter([(
                2.into(),
                PositionCache {
                    edges: Edges {
                        bottom: HashMap::from_iter([(
                            DirectedKey::down(yz, 2),
                            SubLocation::new(xab_yz_id.unwrap(), 1)
                        )]),
                        top: HashSet::from_iter([]),
                    },
                }
            )]),
        }
    );
    assert_eq!(
        aby_found.cache.entries[&yz.index],
        VertexCache {
            index: yz,
            bottom_up: FromIterator::from_iter([]),
            top_down: FromIterator::from_iter([(
                2.into(),
                PositionCache {
                    edges: Edges {
                        bottom: HashMap::from_iter([(
                            DirectedKey::down(y, 2),
                            SubLocation::new(y_z_id.unwrap(), 0)
                        )]),
                        top: HashSet::from_iter([]),
                    },
                }
            )]),
        }
    );
    assert_eq!(aby_found.cache.entries.len(), 5);
    assert_eq!(
        aby_found.kind,
        FinishedKind::Incomplete(EndState {
            reason: EndReason::Mismatch,
            kind: EndKind::Range(RangeEnd {
                root_pos: 2.into(),
                target: DownKey::new(y, 3.into()),
                path: RootedRangePath {
                    root: IndexRoot {
                        location: PatternLocation {
                            parent: xabyz,
                            id: xab_yz_id.unwrap(),
                        },
                    },
                    start: RolePath {
                        sub_path: SubPath {
                            root_entry: 0,
                            path: vec![ChildLocation {
                                parent: xab,
                                pattern_id: x_a_b_id.unwrap(),
                                sub_index: 1,
                            },],
                        },
                        _ty: Default::default(),
                    },
                    end: RolePath {
                        sub_path: SubPath {
                            root_entry: 1,
                            path: vec![ChildLocation {
                                parent: yz,
                                pattern_id: y_z_id.unwrap(),
                                sub_index: 0,
                            },],
                        },
                        _ty: Default::default(),
                    },
                },
            }),
            cursor: PatternPrefixCursor {
                path: RootedRolePath {
                    root: query.clone(),
                    role_path: RolePath {
                        sub_path: SubPath {
                            root_entry: 2,
                            path: vec![],
                        },
                        _ty: Default::default(),
                    },
                },
                relative_pos: 3.into(),
            },
        })
    );
}
