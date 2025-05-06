use std::collections::HashSet;

use crate::insert::ToInsertContext;
use context_search::{
    search::Searchable,
    traversal::result::{
        FinishedKind,
        FinishedState,
    },
};
use context_trace::{
    graph::{
        Hypergraph,
        HypergraphRef,
        getters::vertex::VertexSet,
        kind::BaseGraphKind,
        vertex::{
            child::Child,
            token::Token,
            wide::Wide,
        },
    },
    trace::has_graph::HasGraph,
};
use itertools::*;
use maplit::hashset;
use pretty_assertions::{
    assert_eq,
    assert_matches,
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
    let ab = graph.insert_pattern(vec![a, b]);
    let by = graph.insert_pattern(vec![b, y]);
    let yz = graph.insert_pattern(vec![y, z]);
    let xa = graph.insert_pattern(vec![x, a]);
    let xab = graph.insert_patterns([vec![x, ab], vec![xa, b]]);
    let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
    let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
    print!("{:#?}", xabyz);
    // todo: split sub patterns not caught by query search
    let graph = HypergraphRef::from(graph);
    let query = vec![by, z];
    let byz = graph.insert(query.clone()).expect("Indexing failed");
    assert_eq!(
        byz,
        Child {
            index: 13,
            width: 3.into(),
        },
        "byz"
    );
    let byz_found = graph.find_ancestor(&query);
    assert_matches!(
        byz_found,
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == byz,
        "byz"
    );
    let query = vec![ab, y];
    let aby = graph.insert(query.clone()).expect("Indexing failed");
    let aby_found = graph.find_parent(&query);
    assert_matches!(
        aby_found,
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == aby,
        "aby"
    );
}

#[test]
fn index_pattern2() {
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
    let yz = graph.insert_pattern(vec![y, z]);
    let xab = graph.insert_pattern(vec![x, a, b]);
    let _xyz = graph.insert_pattern(vec![x, yz]);
    let _xabz = graph.insert_pattern(vec![xab, z]);
    let _xabyz = graph.insert_pattern(vec![xab, yz]);

    let graph_ref = HypergraphRef::from(graph);

    let query = vec![a, b, y, x];
    let aby = graph_ref.insert(query.clone()).expect("Indexing failed");
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
    assert_matches!(
        aby_found,
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == aby,
        "aby"
    );
}

#[test]
fn index_infix1() {
    let mut graph =
        context_trace::graph::Hypergraph::<BaseGraphKind>::default();
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
    let yz = graph.insert_pattern(vec![y, z]);
    let xxabyzw = graph.insert_pattern(vec![x, x, a, b, yz, w]);

    let graph_ref = HypergraphRef::from(graph);

    let aby = graph_ref.insert(vec![a, b, y]).expect("Indexing failed");
    let ab = graph_ref
        .find_ancestor(vec![a, b])
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
    assert_matches!(
        aby_found,
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == aby,
        "aby"
    );
    let abyz = graph_ref
        .find_ancestor(vec![ab, yz])
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
    let mut graph =
        context_trace::graph::Hypergraph::<BaseGraphKind>::default();
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
    let yy = graph.insert_pattern(vec![y, y]);
    let xx = graph.insert_pattern(vec![x, x]);
    let xy = graph.insert_pattern(vec![x, y]);
    let xxy = graph.insert_patterns([vec![xx, y], vec![x, xy]]);
    let abcdx = graph.insert_pattern(vec![a, b, c, d, x]);
    let yabcdx = graph.insert_pattern(vec![y, abcdx]);
    let abcdxx = graph.insert_pattern(vec![abcdx, x]);
    let _xxyyabcdxxyy = graph
        .insert_patterns([vec![xx, yy, abcdxx, yy], vec![xxy, yabcdx, xy, y]]);

    let graph_ref = HypergraphRef::from(graph);

    let abcd = graph_ref.insert(vec![a, b, c, d]).expect("Indexing failed");
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
