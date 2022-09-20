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
        Err(NoMatch::SingleIndex),
        "bc"
    );
    let query = b_c_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::complete(query, bc)),
        "b_c"
    );
    let query = a_bc_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::complete(query, abc)),
        "a_bc"
    );
    let query = ab_c_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::complete(query, abc)),
        "ab_c"
    );
    let query = a_bc_d_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::complete(query, abcd)),
        "a_bc_d"
    );
    let query = a_b_c_pattern.clone();
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult::complete(query, abc)),
        "a_b_c"
    );
    let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    assert_eq!(
        graph.find_parent(&query),
        Ok(TraversalResult {
            found: FoundPath::Complete(*abc),
            query: QueryRangePath {
                exit: query.len() - 1,
                query,
                entry: 0,
                start: vec![],
                end: vec![],
            },
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
        Err(NoMatch::SingleIndex),
        "bc"
    );
    let query = b_c_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::complete(query, bc)),
        "b_c"
    );
    let query = a_bc_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::complete(query, abc)),
        "a_bc"
    );
    let query = ab_c_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::complete(query, abc)),
        "ab_c"
    );
    let query = a_bc_d_pattern;
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::complete(query, abcd)),
        "a_bc_d"
    );
    let query = a_b_c_pattern.clone();
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::complete(query, abc)),
        "a_b_c"
    );
    let query =
        vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult::complete(query, ababababcdefghi)),
        "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
    );
    let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    assert_eq!(
        graph.find_ancestor(&query),
        Ok(TraversalResult {
            found: FoundPath::Complete(*abc),
            query: QueryRangePath {
                exit: query.len() - 1,
                query,
                entry: 0,
                start: vec![],
                end: vec![],
            },
        }),
        "a_b_c_c"
    );
}
#[test]
fn find_ancestor2() {
    let mut graph = Hypergraph::default();
    let (a, b, _w, x, y, z) = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ]).into_iter().next_tuple().unwrap();
    let ab = graph.index_pattern([a, b]);
    let by = graph.index_pattern([b, y]);
    let yz = graph.index_pattern([y, z]);
    let xa = graph.index_pattern([x, a]);
    let xab = graph.index_patterns([[x, ab], [xa, b]]);
    let (xaby, xaby_ids) = graph.index_patterns_with_ids([vec![xab, y], vec![xa, by]]);
    let xa_by_id = xaby_ids[1];
    let (xabyz, xabyz_ids) = graph.index_patterns_with_ids([vec![xaby, z], vec![xab, yz]]);
    let xaby_z_id = xabyz_ids[0];
    let graph = HypergraphRef::from(graph);
    let query = vec![by, z];
    let byz_found = graph.find_ancestor(&query);
    assert_eq!(
        byz_found,
        Ok(TraversalResult {
            found: FoundPath::Range(SearchPath {
                start: StartPath::Path {
                    entry: xabyz.to_pattern_location(xaby_z_id)
                        .to_child_location(0),
                    path: vec![
                        ChildLocation {
                            parent: xaby,
                            pattern_id: xa_by_id,
                            sub_index: 1,
                        },
                    ],
                    width: 3,
                    child: by
                },
                end: EndPath {
                    path: vec![],
                    entry: xabyz.to_pattern_location(xaby_z_id)
                        .to_child_location(1),
                    width: 0,
                },
            }),
            query: QueryRangePath::complete(query),
        }),
        "by_z"
    );
}#[test]
fn find_ancestor3() {
    let mut graph = Hypergraph::default();
    let (a, b, _w, x, y, z) = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ]).into_iter().next_tuple().unwrap();
    let ab = graph.index_pattern([a, b]);
    let by = graph.index_pattern([b, y]);
    let yz = graph.index_pattern([y, z]);
    let xa = graph.index_pattern([x, a]);
    let (xab, xab_ids) = graph.index_patterns_with_ids([[x, ab], [xa, b]]);
    let x_ab_id = xab_ids[0];
    let (xaby, xaby_ids) = graph.index_patterns_with_ids([vec![xab, y], vec![xa, by]]);
    let xab_y_id = xaby_ids[0];
    let _xabyz = graph.index_patterns([vec![xaby, z], vec![xab, yz]]);

    let graph = HypergraphRef::from(graph);
    let query = vec![ab, y];
    let aby_found = graph.find_ancestor(&query);
    assert_eq!(
        aby_found,
        Ok(TraversalResult {
            found: FoundPath::Range(SearchPath {
                start: StartPath::Path {
                    entry: xaby.to_pattern_location(xab_y_id)
                        .to_child_location(0),
                    path: vec![
                        ChildLocation {
                            parent: xab,
                            pattern_id: x_ab_id,
                            sub_index: 1,
                        },
                    ],
                    width: 3,
                    child: ab
                },
                end: EndPath {
                    path: vec![],
                    entry: xaby.to_pattern_location(xab_y_id)
                        .to_child_location(1),
                    width: 0,
                },
            }),
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
        ..
     } = &*context();
    assert_eq!(
        graph.find_sequence("a".chars()),
        Err(NoMatch::SingleIndex),
    );
    let query = graph.read().unwrap().expect_token_pattern("abc".chars());
    let abc_found = graph.find_ancestor(&query);
    assert_eq!(
        abc_found,
        Ok(TraversalResult::complete(query, abc)),
        "abc"
    );
    let query = graph.read().unwrap().expect_token_pattern("ababababcdefghi".chars());
    let ababababcdefghi_found = graph.find_ancestor(&query);
    assert_eq!(
        ababababcdefghi_found,
        Ok(TraversalResult::complete(query, ababababcdefghi)),
        "ababababcdefghi"
    );
}