#[allow(clippy::many_single_char_names)]
use super::*;
use crate::*;
use pretty_assertions::assert_eq;
use itertools::*;
use maplit::hashset;
use std::collections::HashSet;
use std::borrow::Borrow;

#[test]
fn index_pattern1() {
    let mut graph = Hypergraph::default();
    let (a, b, _w, x, y, z) = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ]).into_iter().next_tuple().unwrap();
    // index 6
    let ab = graph.index_pattern([a, b]);
    let by = graph.index_pattern([b, y]);
    let yz = graph.index_pattern([y, z]);
    let xa = graph.index_pattern([x, a]);
    let xab = graph.index_patterns([[x, ab], [xa, b]]);
    let xaby = graph.index_patterns([vec![xab, y], vec![xa, by]]);
    let _xabyz = graph.index_patterns([vec![xaby, z], vec![xab, yz]]);
    let graph = HypergraphRef::from(graph);
    let query = vec![by, z];
    let (byz, _) = graph.index_pattern(query.borrow()).expect("Indexing failed");
    assert_eq!(byz, Child {
        index: 13,
        width: 3,
    }, "byz");
    let byz_found = graph.find_ancestor(&query);
    assert_eq!(
        byz_found,
        Ok(TraversalResult::complete(query, byz)),
        "byz"
    );
    let query = vec![ab, y];
    let (aby, _) = graph.index_pattern(query.borrow()).expect("Indexing failed");
    let aby_found = graph.find_parent(&query);
    assert_eq!(
        aby_found,
        Ok(TraversalResult::complete(query, aby)),
        "aby"
    );
}
#[test]
fn index_pattern2() {
    let mut graph = Hypergraph::default();
    let (a, b, _w, x, y, z) = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ]).into_iter().next_tuple().unwrap();
    // index 6
    let yz = graph.index_pattern([y, z]);
    let xab = graph.index_pattern([x, a, b]);
    let _xyz = graph.index_pattern([x, yz]);
    let _xabz = graph.index_pattern([xab, z]);
    let _xabyz = graph.index_pattern([xab, yz]);

    let graph_ref = HypergraphRef::from(graph);

    let query = vec![a, b, y, x];
    let (aby, _) = graph_ref.index_pattern(query.borrow()).expect("Indexing failed");
    assert_eq!(aby, Child {
        index: 12,
        width: 3,
    }, "aby");
    let graph = graph_ref.read().unwrap();
    let aby_vertex = graph.expect_vertex_data(aby);
    assert_eq!(aby_vertex.parents.len(), 1, "aby");
    assert_eq!(aby_vertex.children.len(), 1, "aby");
    drop(graph);
    let query = vec![a, b, y];
    let aby_found = graph_ref.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(TraversalResult {
            found: FoundPath::Complete(aby),
            query: QueryRangePath::complete(query),
        }),
        "aby"
    );
}
#[test]
fn index_infix1() {
    let mut graph = Hypergraph::default();
    let (a, b, w, x, y, z) = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ]).into_iter().next_tuple().unwrap();
    // index 6
    let yz = graph.index_pattern([y, z]);
    let xxabyzw = graph.index_pattern([x, x, a, b, yz, w]);

    let graph_ref = HypergraphRef::from(graph);

    let (aby, _) = graph_ref.index_pattern([a, b, y]).expect("Indexing failed");
    let ab = graph_ref.find_ancestor([a, b]).unwrap().expect_complete("ab");
    let graph = graph_ref.read().unwrap();
    let aby_vertex = graph.expect_vertex_data(aby);
    assert_eq!(aby.width, 3, "aby");
    assert_eq!(aby_vertex.parents.len(), 1, "aby");
    assert_eq!(aby_vertex.children.len(), 1, "aby");
    assert_eq!(
        aby_vertex.get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![ab, y]
        ],
        "aby"
    );
    drop(graph);
    let query = vec![a, b, y];
    let aby_found = graph_ref.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(TraversalResult {
            found: FoundPath::Complete(aby),
            query: QueryRangePath::complete(query),
        }),
        "aby"
    );
    let abyz = graph_ref.find_ancestor([ab, yz]).unwrap().expect_complete("abyz");
    let graph = graph_ref.read().unwrap();
    let abyz_vertex = graph.expect_vertex_data(abyz);
    assert_eq!(
        abyz_vertex.get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![aby, z],
            vec![ab, yz]
        ],
        "abyz"
    );
    let xxabyzw_vertex = graph.expect_vertex_data(xxabyzw);
    assert_eq!(
        xxabyzw_vertex.get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![x, x, abyz, w]
        ],
        "xxabyzw"
    );
}