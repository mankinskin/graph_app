use std::collections::HashSet;

use crate::context::has_read_context::HasReadCtx;
use context_search::{
    fold::result::{
        FinishedKind,
        FinishedState,
    },
    search::Searchable,
};
use context_trace::{
    assert_indices,
    assert_not_indices,
    assert_patterns,
    expect_tokens,
    graph::{
        vertex::{
            has_vertex_data::HasVertexData,
            parent::PatternIndex,
            wide::Wide,
        },
        Hypergraph,
        HypergraphRef,
    },
    tests::assert_parents,
    trace::has_graph::HasGraph,
};
use maplit::hashset;
use pretty_assertions::{
    assert_eq,
    assert_matches,
};
#[test]
fn sync_read_text1() {
    let mut graph: HypergraphRef = HypergraphRef::from(Hypergraph::default());
    let result = (&mut graph, "heldldo world!".chars())
        .read_sequence()
        .unwrap();
    expect_tokens!(graph, {h, e, l, d, o, w, r});
    let g = graph.graph();
    let space = g.expect_token_child(' ');
    let exclam = g.expect_token_child('!');
    drop(g);

    assert_indices!(graph, ld);
    assert_patterns!(
        graph,
        ld => [[l, d]]
    );
    assert_patterns!(
        graph,
        result => [[h, e, ld, ld, o, space, w, o, r, ld, exclam]]
    );
}
#[test]
fn sync_read_text2() {
    let mut graph = HypergraphRef::default();
    let heldld = (&mut graph, "heldld".chars()).read_sequence().unwrap();
    expect_tokens!(graph, {h, e, l, d});
    assert_indices!(graph, ld);
    assert_not_indices!(graph, held, he, hel);
    assert_patterns!(
        graph,
        ld => [[l, d]],
        heldld => [[h, e, ld, ld]]
    );

    let hell = (&mut graph, "hell".chars()).read_sequence().unwrap();

    assert_indices!(graph, he, hel, held);
    assert_patterns! {
        graph,
        he => [[h, e]],
        hel => [[he, l]],
        hell => [[hel, l]],
        held => [[hel, d], [he, ld]],
        heldld => [[held, ld]]
    };
}

#[test]
fn read_sequence1() {
    let mut graph = HypergraphRef::default();
    let ind_hypergraph =
        (&mut graph, "hypergraph".chars()).read_sequence().unwrap();

    expect_tokens!(graph, {h, y, p, e, r, g, a});
    {
        assert_patterns! {
            graph,
            ind_hypergraph => [[h, y, p, e, r, g, r, a, p, h]]
        };
        let gr = graph.graph();
        let pid = *ind_hypergraph
            .vertex(&gr)
            .get_child_patterns()
            .iter()
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
            hyper => [[h, y, p, e, r]]
        };
        assert_eq!(hyper.width(), 5);
        assert_patterns! {
            gr,
            ind_hypergraph => [[hyper, g, r, a, p, h]]
        };
        assert_eq!(ind_hypergraph.width(), 10);
    }
    let ind_graph = (&mut graph, "graph".chars()).read_sequence().unwrap();
    assert_patterns! {
        graph,
        ind_graph => [[g, r, a, p, h]]
    };
    assert_eq!(ind_graph.width(), 5);
    assert_patterns! {
        graph,
        ind_hypergraph => [[hyper, ind_graph]]
    };
    assert_eq!(ind_hypergraph.width(), 10);
}
#[test]
fn read_sequence2() {
    let mut graph = HypergraphRef::default();
    let ind_abab = (&mut graph, "abab".chars()).read_sequence().unwrap();
    expect_tokens!(graph, {a, b});
    assert_indices!(graph, ab);
    {
        assert_patterns! {
            graph,
            ind_abab => [[ab, ab]]
        };
        let gr = graph.graph();
        let pid = *ab.vertex(&gr).get_child_patterns().iter().next().unwrap().0;
        assert_parents(&gr, a, ab, [PatternIndex::new(pid, 0)]);
        assert_parents(&gr, b, ab, [PatternIndex::new(pid, 1)]);
        let pid = *ind_abab
            .vertex(&gr)
            .get_child_patterns()
            .iter()
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
    expect_tokens!(graph, {s, u, b, d, i, v, o, n});
    {
        assert_patterns! {
            graph,
            subdivision => [[s, u, b, d, i, v, i, s, i, o, n]]
        };
        let graph = graph.graph();
        let pid = *subdivision
            .vertex(&graph)
            .get_child_patterns()
            .iter()
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
        expect_tokens!(graph, {a, l, z, t});

        assert_indices!(graph, vis, su, vi, visu, ion);
        assert_patterns! {
            graph,
            su => [[s, u]],
            vi => [[v, i]],
            vis => [[vi, s]],
            visu => [[vis, u], [vi, su]],
            ion => [[i, o, n]],
            visualization => [[visu, a, l, i, z, a, t, ion]],
            subdivision => [[su, b, d, i, vis, ion]]
        };
    }
}

#[test]
fn read_infix2() {
    let mut graph = HypergraphRef::default();
    let subvisu = (&mut graph, "subvisu".chars()).read_sequence().unwrap();
    assert_eq!(subvisu.width(), 7);
    expect_tokens!(graph, {s, u, b, v, i});

    assert_indices!(graph, su);
    assert_patterns! {
        graph,
        su => [[s, u]],
        subvisu => [[su, b, v, i, su]]
    };

    let visub = (&mut graph, "visub".chars()).read_sequence().unwrap();
    //let visub_patterns = visub.expect_child_patterns(&graph);
    //println!("{:#?}", graph.graph().pattern_strings(visub_patterns.values()));
    assert_eq!(visub.width(), 5);
    assert_indices!(graph, vi, sub, visu);
    assert_patterns! {
        graph,
        su => [[s, u]],
        subvisu => [[su, b, v, i, su]],
        sub => [[su, b]],
        visu => [[vi, su]],
        visub => [[visu, b], [vi, sub]],
        subvisu => [[visu, b], [vi, sub]]
    };
}

#[test]
fn read_loose_sequence1() {
    let mut graph = HypergraphRef::default();
    let abxaxxb = (&mut graph, "abxaxxb".chars()).read_sequence().unwrap();
    assert_eq!(abxaxxb.width(), 7);
    expect_tokens!(graph, {a, b, x});

    assert_patterns! {
        graph,
        abxaxxb => [
            [a, b, x, a, x, x, b]]
    };
}

#[test]
fn read_repeating_known1() {
    let mut graph = HypergraphRef::default();
    let xyyxy = (&mut graph, "xyyxy".chars()).read_sequence().unwrap();
    assert_eq!(xyyxy.width(), 5);
    expect_tokens!(graph, {x, y});
    assert_indices!(graph, xy);
    assert_patterns! {
        graph,
        xy => [[x, y]],
        xyyxy => [[xy, y, xy]]
    };
}

#[test]
fn read_multiple_overlaps1() {
    let mut graph = HypergraphRef::default();
    let abcde = (&mut graph, "abcde".chars()).read_sequence().unwrap();
    // abcde
    //  bcde
    //  bcdea

    expect_tokens!(graph, {a, b, c, d, e});
    assert_patterns! {
        graph,
        abcde => [[a, b, c, d, e]]
    };
    let bcdea = (&mut graph, "bcdea".chars()).read_sequence().unwrap();
    assert_indices!(graph, bcde);
    assert_patterns! {
        graph,
        bcde => [[b, c, d, e]],
        bcdea => [[bcde, a]],
        abcde => [[a, bcde]]
    };

    let cdeab = (&mut graph, "cdeab".chars()).read_sequence().unwrap();
    // abcde
    //  bcde
    //  bcdea
    //   cdea
    //   cdeab
    assert_indices!(graph, cde, ab, cdea);
    assert_patterns! {
        graph,
        ab => [[a, b]],
        cde => [[c, d, e]],
        cdea => [[cde, a]],
        bcde => [[b, cde]],
        abcde => [[a, bcde], [ab, cde]],
        bcdea => [[bcde, a], [b, cdea]],
        cdeab => [[cde, ab], [cdea, b]]
    };
    let deabc = (&mut graph, "deabc".chars()).read_sequence().unwrap();

    assert_indices!(graph, de, dea, bc, deab, abc);
    assert_patterns! {
        graph,
        de => [[d, e]],
        cde => [[c, de]],
        dea => [[de, a]],
        cdea => [[cde, a], [c, dea]],
        bc => [[b, c]],
        bcde => [[b, cde], [bc, de]],
        bcdea => [[bcde, a], [b, cdea], [bc, dea]],
        deab => [[de, ab], [dea, b]],
        abc => [[ab, c], [a, bc]],
        abcde => [[abc, de], [a, bcde], [ab, cde]],
        deabc => [[de, abc], [dea, bc], [deab, c]]
    };
    let eabcd = (&mut graph, "eabcd".chars()).read_sequence().unwrap();
    assert_indices!(graph, abcd, bcd, cd);
    assert_patterns! {
            graph,
            cd => [[c, d]],
            bcd => [[b, cd], [bc, d]],
            abcd => [[abc, d], [a, bcd]]
    };
    let abcdeabcde =
        (&mut graph, "abcdeabcde".chars()).read_sequence().unwrap();
    assert_patterns! {
        graph,
        abcdeabcde =>
            [
                [abcde, abcde],
                [a, bcdea, bcde],
                [ab, cdeab, cde],
                [abc, deabc, de],
                [abcd, eabcd, e],
            ]
    };
}
