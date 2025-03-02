#[cfg(test)]
use {
    crate::graph::{
        kind::BaseGraphKind,
        vertex::token::Token,
        Hypergraph,
    },
    itertools::Itertools,
};

#[test]
fn test_to_petgraph() {
    let mut graph = Hypergraph::<BaseGraphKind>::default();
    let (a, b, c, d) = graph
        .insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])
        .into_iter()
        .next_tuple()
        .unwrap();
    // ab cd
    // abc d
    // a bcd

    let ab = graph.insert_pattern(vec![a, b]);
    let bc = graph.insert_pattern(vec![b, c]);
    let abc = graph.insert_patterns([vec![ab, c], vec![a, bc]]);
    let cd = graph.insert_pattern(vec![c, d]);
    let bcd = graph.insert_patterns([vec![bc, d], vec![b, cd]]);
    let _abcd = graph.insert_patterns([vec![abc, d], vec![a, bcd]]);
    let pg = graph.to_petgraph();
    pg.write_to_file("assets/test_graph1.dot")
        .expect("Failed to write assets/test_graph1.dot file!");
}
