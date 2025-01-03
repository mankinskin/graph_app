use std::{
    borrow::Borrow,
    collections::HashSet,
};

use itertools::*;
use maplit::hashset;
use pretty_assertions::assert_eq;

use crate::{insert::HasInsertContext, search::Searchable};
use hypercontext_api::{
    graph::{
        getters::vertex::VertexSet,
        kind::BaseGraphKind,
        vertex::{
            child::Child,
            token::Token,
            wide::Wide,
        },
        Hypergraph,
        HypergraphRef,
    },
    path::structs::query_range_path::{
        QueryPath,
        QueryRangePath,
    },
    traversal::{
        result::{FinishedState, FoundRange}, state::query::QueryState, traversable::Traversable
    },
};

#[test]
fn index_pattern1() {
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
    // index 6
    let ab = graph.insert_pattern([a, b]);
    let by = graph.insert_pattern([b, y]);
    let yz = graph.insert_pattern([y, z]);
    let xa = graph.insert_pattern([x, a]);
    let xab = graph.insert_patterns([[x, ab], [xa, b]]);
    let xaby = graph.insert_patterns([[xab, y], [xa, by]]);
    let _xabyz = graph.insert_patterns([[xaby, z], [xab, yz]]);
    // todo: split sub patterns not caught by query search
    let graph = HypergraphRef::from(graph);
    let query = vec![by, z];
    let (byz, _) = graph
        .index_pattern(query.borrow())
        .expect("Indexing failed");
    assert_eq!(
        byz,
        Child {
            index: 13,
            width: 3.into(),
        },
        "byz"
    );
    let byz_found = graph.find_ancestor(&query);
    assert_eq!(
        byz_found,
        Ok(FinishedState::new_complete(query, byz)),
        "byz"
    );
    let query = vec![ab, y];
    let (aby, _) = graph
        .index_pattern(query.borrow())
        .expect("Indexing failed");
    let aby_found = graph.find_parent(&query);
    assert_eq!(
        aby_found,
        Ok(FinishedState::new_complete(query, aby)),
        "aby"
    );
}

#[test]
fn index_pattern2() {
    let mut graph = hypercontext_api::graph::Hypergraph::<BaseGraphKind>::default();
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
    let yz = graph.insert_pattern([y, z]);
    let xab = graph.insert_pattern([x, a, b]);
    let _xyz = graph.insert_pattern([x, yz]);
    let _xabz = graph.insert_pattern([xab, z]);
    let _xabyz = graph.insert_pattern([xab, yz]);

    let graph_ref = HypergraphRef::from(graph);

    let query = vec![a, b, y, x];
    let (aby, _) = graph_ref
        .index_pattern(query.borrow())
        .expect("Indexing failed");
    assert_eq!(aby.width(), 3);
    let ab = graph_ref
        .find_sequence("ab".chars())
        .unwrap()
        .expect_complete("ab");

    let graph = graph_ref.graph();
    let aby_vertex = graph.expect_vertex(aby);
    assert_eq!(aby_vertex.parents.len(), 1, "aby");
    assert_eq!(
        aby_vertex
            .get_child_pattern_set()
            .into_iter()
            .collect::<HashSet<_>>(),
        hashset![vec![ab, y],]
    );
    drop(graph);
    let query = vec![a, b, y];
    let aby_found = graph_ref.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(FinishedState {
            result: FoundRange::Complete(aby),
            query: QueryState {
                pos: (query.len() - 1).into(),
                path: QueryRangePath::complete(query),
            },
        }),
        "aby"
    );
}

#[test]
fn index_infix1() {
    let mut graph = hypercontext_api::graph::Hypergraph::<BaseGraphKind>::default();
    let (a, b, w, x, y, z) = graph
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
    let yz = graph.insert_pattern([y, z]);
    let xxabyzw = graph.insert_pattern([x, x, a, b, yz, w]);

    let graph_ref = HypergraphRef::from(graph);

    let (aby, _) = graph_ref.index_pattern([a, b, y]).expect("Indexing failed");
    let ab = graph_ref
        .find_ancestor([a, b])
        .unwrap()
        .expect_complete("ab");
    let graph = graph_ref.graph();
    let aby_vertex = graph.expect_vertex(aby);
    assert_eq!(aby.width(), 3, "aby");
    assert_eq!(aby_vertex.parents.len(), 1, "aby");
    assert_eq!(aby_vertex.children.len(), 1, "aby");
    assert_eq!(
        aby_vertex
            .get_child_pattern_set()
            .into_iter()
            .collect::<HashSet<_>>(),
        hashset![vec![ab, y]],
        "aby"
    );
    drop(graph);
    let query = vec![a, b, y];
    let aby_found = graph_ref.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(FinishedState {
            result: FoundRange::Complete(aby),
            query: QueryState {
                pos: (query.len() - 1).into(),
                path: QueryRangePath::complete(query),
            }
        }),
        "aby"
    );
    let abyz = graph_ref
        .find_ancestor([ab, yz])
        .unwrap()
        .expect_complete("abyz");
    let graph = graph_ref.graph();
    let abyz_vertex = graph.expect_vertex(abyz);
    assert_eq!(
        abyz_vertex
            .get_child_pattern_set()
            .into_iter()
            .collect::<HashSet<_>>(),
        hashset![vec![aby, z], vec![ab, yz]],
        "abyz"
    );
    let xxabyzw_vertex = graph.expect_vertex(xxabyzw);
    assert_eq!(
        xxabyzw_vertex
            .get_child_pattern_set()
            .into_iter()
            .collect::<HashSet<_>>(),
        hashset![vec![x, x, abyz, w]],
        "xxabyzw"
    );
}

#[test]
fn index_infix2() {
    let mut graph = hypercontext_api::graph::Hypergraph::<BaseGraphKind>::default();
    let (a, b, c, d, x, y) = graph
        .insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
            Token::Element('x'),
            Token::Element('y'),
            //Token::Element('z'),
        ])
        .into_iter()
        .next_tuple()
        .unwrap();
    // index 6
    let yy = graph.insert_pattern([y, y]);
    let xx = graph.insert_pattern([x, x]);
    let xy = graph.insert_pattern([x, y]);
    let xxy = graph.insert_patterns([[xx, y], [x, xy]]);
    let abcdx = graph.insert_pattern([a, b, c, d, x]);
    let yabcdx = graph.insert_pattern([y, abcdx]);
    let abcdxx = graph.insert_pattern([abcdx, x]);
    let _xxyyabcdxxyy = graph.insert_patterns([[xx, yy, abcdxx, yy], [xxy, yabcdx, xy, y]]);

    let graph_ref = HypergraphRef::from(graph);

    let (abcd, _) = graph_ref
        .index_pattern([a, b, c, d])
        .expect("Indexing failed");
    let graph = graph_ref.graph();
    let abcd_vertex = graph.expect_vertex(abcd);
    assert_eq!(abcd.width(), 4, "abcd");
    assert_eq!(abcd_vertex.parents.len(), 1, "abcd");
    assert_eq!(abcd_vertex.children.len(), 1, "abcd");
    assert_eq!(
        abcd_vertex
            .get_child_pattern_set()
            .into_iter()
            .collect::<HashSet<_>>(),
        hashset![vec![a, b, c, d]],
        "abc"
    );
    drop(graph);
    let graph = graph_ref.graph();
    let abcdx_vertex = graph.expect_vertex(abcdx);
    assert_eq!(
        abcdx_vertex
            .get_child_pattern_set()
            .into_iter()
            .collect::<HashSet<_>>(),
        hashset![vec![abcd, x],],
        "abcx"
    );
}
