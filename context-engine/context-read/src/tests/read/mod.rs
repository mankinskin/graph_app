use std::{
    collections::HashSet,
    iter::FromIterator,
};

use crate::context::has_read_context::HasReadCtx;
use charify::charify;
use context_search::search::Searchable;
use context_trace::{
    graph::{
        vertex::{
            has_vertex_data::HasVertexData,
            has_vertex_index::ToChild,
            parent::{
                Parent,
                PatternIndex,
            },
            wide::Wide,
        },
        Hypergraph,
        HypergraphRef,
    },
    trace::has_graph::HasGraph,
    HashMap,
};
use maplit::hashset;
use pretty_assertions::assert_eq;

macro_rules! assert_patterns {
    ($graph:ident, $($name:ident => $pats:expr),*) => {
        $(
            let g = $graph.graph();
            let pats: HashSet<_> =
                $name.vertex(&g).get_child_pattern_set().into_iter().collect();
            assert_eq!(pats, $pats);
            #[allow(dropping_references)]
            drop(g);
        )*
    };
}
macro_rules! find_index {
    ($graph:ident, $name:ident) => {
        let $name = $graph
            .find_sequence(stringify!($name).chars())
            .unwrap()
            .expect_complete(stringify!($name));
    };
}
macro_rules! expect_tokens {
    ($graph:ident, {$($name:ident),*}) => {

        $(let $name = $graph.expect_token_child(charify!($name));)*
    };
}
fn assert_parents(
    graph: &Hypergraph,
    child: impl ToChild,
    parent: impl ToChild,
    pattern_indices: impl IntoIterator<Item = PatternIndex>,
) {
    assert_eq!(
        graph
            .expect_parents(child)
            .clone()
            .into_iter()
            .collect::<HashMap<_, _>>(),
        HashMap::from_iter([(
            parent.vertex_index(),
            Parent {
                pattern_indices: pattern_indices.into_iter().collect(),
                width: parent.width(),
            }
        )])
    );
}

#[test]
fn sync_read_text1() {
    let mut graph: HypergraphRef = HypergraphRef::from(Hypergraph::default());
    let result = (&mut graph, "heldldo world!".chars())
        .read_sequence()
        .unwrap();
    let g = graph.graph();
    expect_tokens!(g, {h, e, l, d, o, w, r});
    let space = g.expect_token_child(' ');
    let exclam = g.expect_token_child('!');
    drop(g);

    find_index!(graph, ld);
    assert_patterns!(
        graph,
        ld => hashset![vec![l, d],]
    );
    assert_patterns!(
        graph,
        result => hashset![vec![h, e, ld, ld, o, space, w, o, r, ld, exclam],]
    );
}
#[test]
fn sync_read_text2() {
    let mut graph = HypergraphRef::default();
    let heldld = (&mut graph, "heldld".chars()).read_sequence().unwrap();
    let g = graph.graph();
    expect_tokens!(g, {h, e, l, d});
    drop(g);
    find_index!(graph, ld);
    assert_patterns!(
        graph,
        ld => hashset![vec![l, d],]
    );
    assert_patterns!(
        graph,
        heldld => hashset![vec![h, e, ld, ld],]
    );

    let hell = (&mut graph, "hell".chars()).read_sequence().unwrap();

    find_index!(graph, he);
    find_index!(graph, hel);
    find_index!(graph, held);
    assert_patterns! {
        graph,
        he => hashset![vec![h, e],],
        hel => hashset![vec![he, l],],
        hell => hashset![vec![hel, l],],
        held => hashset![vec![hel, d], vec![he, ld],],
        heldld => hashset![vec![held, ld]]
    };
}

#[test]
fn read_sequence1() {
    let mut graph = HypergraphRef::default();
    let ind_hypergraph =
        (&mut graph, "hypergraph".chars()).read_sequence().unwrap();

    let gr = graph.graph();
    expect_tokens!(gr, {h, y, p, e, r, g, a});
    drop(gr);
    {
        assert_patterns! {
            graph,
            ind_hypergraph => hashset![vec![h, y, p, e, r, g, r, a, p, h],]
        };
        let gr = graph.graph();
        let pid = *ind_hypergraph
            .vertex(&gr)
            .get_child_patterns()
            .into_iter()
            .next()
            .unwrap()
            .0;
        assert_parents(
            &gr,
            h,
            ind_hypergraph,
            [PatternIndex::new(pid, 0), PatternIndex::new(pid, 9)],
        );
        assert_parents(&gr, y, ind_hypergraph, [PatternIndex::new(pid, 1)]);
        assert_parents(
            &gr,
            p,
            ind_hypergraph,
            [PatternIndex::new(pid, 2), PatternIndex::new(pid, 8)],
        );
        assert_parents(&gr, e, ind_hypergraph, [PatternIndex::new(pid, 3)]);
        assert_parents(
            &gr,
            r,
            ind_hypergraph,
            [PatternIndex::new(pid, 6), PatternIndex::new(pid, 4)],
        );
        assert_parents(&gr, a, ind_hypergraph, [PatternIndex::new(pid, 7)]);
        assert_eq!(ind_hypergraph.width(), 10);
    }
    let hyper = (&mut graph, "hyper".chars()).read_sequence().unwrap();
    {
        let gr = graph.graph();
        assert_patterns! {
            gr,
            hyper => hashset![vec![h, y, p, e, r],]
        };
        assert_eq!(hyper.width(), 5);
        assert_patterns! {
            gr,
            ind_hypergraph => hashset![vec![hyper, g, r, a, p, h],]
        };
        assert_eq!(ind_hypergraph.width(), 10);
    }
    let ind_graph = (&mut graph, "graph".chars()).read_sequence().unwrap();
    assert_patterns! {
        graph,
        ind_graph => hashset![vec![g, r, a, p, h],]
    };
    assert_eq!(ind_graph.width(), 5);
    assert_patterns! {
        graph,
        ind_hypergraph => hashset![vec![hyper, ind_graph],]
    };
    assert_eq!(ind_hypergraph.width(), 10);
}
#[test]
fn read_sequence2() {
    let mut graph = HypergraphRef::default();
    let ind_abab = (&mut graph, "abab".chars()).read_sequence().unwrap();
    let gr = graph.graph();
    expect_tokens!(gr, {a, b});
    drop(gr);
    let ab = graph
        .find_ancestor(vec![a, b])
        .unwrap()
        .expect_complete("ab");
    {
        assert_patterns! {
            graph,
            ind_abab => hashset![vec![ab, ab],]
        };
        let gr = graph.graph();
        let pid = *ab
            .vertex(&gr)
            .get_child_patterns()
            .into_iter()
            .next()
            .unwrap()
            .0;
        assert_parents(&gr, a, ab, [PatternIndex::new(pid, 0)]);
        assert_parents(&gr, b, ab, [PatternIndex::new(pid, 1)]);
        let pid = *ind_abab
            .vertex(&gr)
            .get_child_patterns()
            .into_iter()
            .next()
            .unwrap()
            .0;
        assert_parents(
            &gr,
            ab,
            ind_abab,
            [PatternIndex::new(pid, 0), PatternIndex::new(pid, 1)],
        );
    }
    let ind_a = (graph, "a".chars()).read_sequence().unwrap();
    assert_eq!(ab.width(), 2);
    assert_eq!(ind_abab.width(), 4);
    assert_eq!(ind_a, a);
}

#[test]
fn read_infix1() {
    let mut graph = HypergraphRef::default();
    let subdivision =
        (&mut graph, "subdivision".chars()).read_sequence().unwrap();
    assert_eq!(subdivision.width(), 11);
    let g = graph.graph();
    expect_tokens!(g, {s, u, b, d, i, v, o, n});
    drop(g);
    {
        let graph = graph.graph();
        assert_patterns! {
            graph,
            subdivision => hashset![vec![s, u, b, d, i, v, i, s, i, o, n],]
        };
        let pid = *subdivision
            .vertex(&graph)
            .get_child_patterns()
            .into_iter()
            .next()
            .unwrap()
            .0;
        assert_parents(
            &graph,
            s,
            subdivision,
            [PatternIndex::new(pid, 0), PatternIndex::new(pid, 7)],
        );
        assert_parents(&graph, u, subdivision, [PatternIndex::new(pid, 1)]);
        assert_parents(&graph, b, subdivision, [PatternIndex::new(pid, 2)]);
        assert_parents(&graph, d, subdivision, [PatternIndex::new(pid, 3)]);
        assert_parents(
            &graph,
            i,
            subdivision,
            [
                PatternIndex::new(pid, 4),
                PatternIndex::new(pid, 6),
                PatternIndex::new(pid, 8),
            ],
        );
        assert_parents(&graph, v, subdivision, [PatternIndex::new(pid, 5)]);
        assert_parents(&graph, o, subdivision, [PatternIndex::new(pid, 9)]);
        assert_parents(&graph, n, subdivision, [PatternIndex::new(pid, 10)]);
    }
    let visualization = (&mut graph, "visualization".chars())
        .read_sequence()
        .unwrap();
    {
        let g = graph.graph();
        expect_tokens!(g, {a, l, z, t});
        drop(g);

        find_index!(graph, vis);
        find_index!(graph, su);
        find_index!(graph, vi);
        find_index!(graph, visu);
        find_index!(graph, ion);
        assert_patterns! {
            graph,
            su => hashset![vec![s, u],],
            vi => hashset![vec![v, i],],
            vis => hashset![vec![vi, s],],
            visu => hashset![vec![vis, u], vec![vi, su],],
            ion => hashset![vec![i, o, n],],
            visualization => hashset![vec![visu, a, l, i, z, a, t, ion],],
            subdivision => hashset![vec![su, b, d, i, vis, ion],]
        };
    }
}

#[test]
fn read_infix2() {
    let mut graph = HypergraphRef::default();
    let subvisu = (&mut graph, "subvisu".chars()).read_sequence().unwrap();
    assert_eq!(subvisu.width(), 7);
    let g = graph.graph();
    expect_tokens!(g, {s, u, b, v, i});
    drop(g);

    find_index!(graph, su);
    assert_patterns! {
        graph,
        su => hashset![vec![s, u],],
        subvisu => hashset![vec![su, b, v, i, su],]
    };

    let visub = (&mut graph, "visub".chars()).read_sequence().unwrap();
    //let visub_patterns = visub.expect_child_patterns(&graph);
    //println!("{:#?}", graph.graph().pattern_strings(visub_patterns.values()));
    assert_eq!(visub.width(), 5);
    find_index!(graph, vi);
    find_index!(graph, sub);
    find_index!(graph, visu);
    assert_patterns! {
        graph,
        su => hashset![vec![s, u],],
        subvisu => hashset![vec![su, b, v, i, su],],
        sub => hashset![vec![su, b],],
        visu => hashset![vec![vi, su],],
        visub => hashset![vec![visu, b], vec![vi, sub],],
        subvisu => hashset![vec![visu, b], vec![vi, sub],]
    };
}

#[test]
fn read_loose_sequence1() {
    let mut graph = HypergraphRef::default();
    let abxaxxb = (&mut graph, "abxaxxb".chars()).read_sequence().unwrap();
    assert_eq!(abxaxxb.width(), 7);
    let g = graph.graph();
    expect_tokens!(g, {a, b, x});

    assert_patterns! {
        graph,
        abxaxxb => hashset![vec![a, b, x, a, x, x, b],]
    };
}

#[test]
fn read_repeating_known1() {
    let mut graph = HypergraphRef::default();
    let xyyxy = (&mut graph, "xyyxy".chars()).read_sequence().unwrap();
    assert_eq!(xyyxy.width(), 5);
    let g = graph.graph();
    expect_tokens!(g, {x, y});

    drop(g);
    find_index!(graph, xy);
    assert_patterns! {
        graph,
        xy => hashset![vec![x, y],],
        xyyxy => hashset![vec![xy, y, xy],]
    };
}

#[test]
fn read_multiple_overlaps1() {
    let mut graph = HypergraphRef::default();
    let abcde = (&mut graph, "abcde".chars()).read_sequence().unwrap();
    // abcde
    //  bcde
    //  bcdea

    let g = graph.graph();
    expect_tokens!(g, {a, b, c, d, e});
    assert_patterns! {
        graph,
        abcde => hashset![vec![a, b, c, d, e],]
    };
    drop(g);
    let bcdea = (&mut graph, "bcdea".chars()).read_sequence().unwrap();
    find_index!(graph, bcde);
    assert_patterns! {
        graph,
        bcde => hashset![vec![b, c, d, e],],
        bcdea => hashset![vec![bcde, a],],
        abcde => hashset![vec![a, bcde],]
    };

    let cdeab = (&mut graph, "cdeab".chars()).read_sequence().unwrap();
    // abcde
    //  bcde
    //  bcdea
    //   cdea
    //   cdeab
    find_index!(graph, cde);
    find_index!(graph, ab);
    find_index!(graph, cdea);
    assert_patterns! {
        graph,
        ab => hashset![vec![a, b],],
        cde => hashset![vec![c, d, e],],
        cdea => hashset![vec![cde, a],],
        bcde => hashset![vec![b, cde],],
        abcde => hashset![vec![a, bcde], vec![ab, cde],],
        bcdea => hashset![vec![bcde, a], vec![b, cdea],],
        cdeab => hashset![vec![cde, ab], vec![cdea, b],]
    };
    let deabc = (&mut graph, "deabc".chars()).read_sequence().unwrap();

    find_index!(graph, de);
    find_index!(graph, dea);
    find_index!(graph, bc);
    find_index!(graph, deab);
    find_index!(graph, abc);
    assert_patterns! {
        graph,
        de => hashset![vec![d, e],],
        cde => hashset![vec![c, de],],
        dea => hashset![vec![de, a],],
        cdea => hashset![vec![cde, a], vec![c, dea],],
        bc => hashset![vec![b, c],],
        bcde => hashset![vec![b, cde], vec![bc, de],],
        bcdea => hashset![vec![bcde, a], vec![b, cdea], vec![bc, dea],],
        deab => hashset![vec![de, ab], vec![dea, b],],
        abc => hashset![vec![ab, c], vec![a, bc],],
        abcde => hashset![vec![abc, de], vec![a, bcde], vec![ab, cde],],
        deabc => hashset![vec![de, abc], vec![dea, bc], vec![deab, c],]
    };
    let eabcd = (&mut graph, "eabcd".chars()).read_sequence().unwrap();
    find_index!(graph, abcd);
    find_index!(graph, bcd);
    find_index!(graph, cd);
    assert_patterns! {
            graph,
            cd => hashset![vec![c, d],],
            bcd => hashset![vec![b, cd], vec![bc, d],],
            abcd => hashset![vec![abc, d], vec![a, bcd],]
    };
    let abcdeabcde =
        (&mut graph, "abcdeabcde".chars()).read_sequence().unwrap();
    assert_patterns! {
        graph,
        abcdeabcde =>
            hashset![
                vec![abcde, abcde],
                vec![a, bcdea, bcde],
                vec![ab, cdeab, cde],
                vec![abc, deabc, de],
                vec![abcd, eabcd, e],
            ]
    };
}
