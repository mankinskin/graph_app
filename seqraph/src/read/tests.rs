use crate::*;
//use tokio::sync::mpsc;
//use tokio_stream::wrappers::*;
use maplit::{hashset, hashmap};
use std::collections::HashSet;
use pretty_assertions::assert_eq;

fn assert_child_of_at<T: Tokenize>(graph: &Hypergraph<T>, child: impl AsChild, parent: impl AsChild, pattern_indices: impl IntoIterator<Item=PatternIndex>) {
    assert_eq!(graph.expect_parents(child).clone().into_iter().collect::<HashMap<_, _>>(), hashmap![
        parent.index() => Parent {
            pattern_indices: pattern_indices.into_iter().collect(),
            width: parent.width(),
        },
    ]);
}

#[test]
fn sync_read_text1() {
    let mut graph: HypergraphRef<char> = HypergraphRef::from(Hypergraph::default());
    let result = graph.read_sequence("heldldo world!".chars()).unwrap();
    let g = graph.graph();
    let h = g.expect_token_child('h');
    let e = g.expect_token_child('e');
    let l = g.expect_token_child('l');
    let d = g.expect_token_child('d');
    let o = g.expect_token_child('o');
    let space = g.expect_token_child(' ');
    let w = g.expect_token_child('w');
    let r = g.expect_token_child('r');
    let exclam = g.expect_token_child('!');
    drop(g);
    let ld = graph.find_sequence("ld".chars()).unwrap().expect_complete("ld");
    let g = graph.graph();
    let pats: HashSet<_> = ld.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![l, d],
    ]);
    let pats: HashSet<_> = result.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![h, e, ld, ld, o, space, w, o, r, ld, exclam],
    ]);
}

#[test]
fn sync_read_text2() {
    let mut graph = HypergraphRef::default();
    let heldld = graph.read_sequence("heldld".chars()).unwrap();
    let g = graph.graph();
    let h = g.expect_token_child('h');
    let e = g.expect_token_child('e');
    let l = g.expect_token_child('l');
    let d = g.expect_token_child('d');
    drop(g);
    let ld = graph.find_sequence("ld".chars()).unwrap().expect_complete("ld");
    let g = graph.graph();
    let pats: HashSet<_> = ld.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![l, d],
    ]);
    let pats: HashSet<_> = heldld.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![h, e, ld, ld],
    ]);
    drop(g);
    let hell = graph.read_sequence("hell".chars()).unwrap();
    let he = graph.find_sequence("he".chars()).unwrap().expect_complete("he");
    let g = graph.graph();
    let pats: HashSet<_> = he.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![h, e],
    ]);
    drop(g);
    let hel = graph.find_sequence("hel".chars()).unwrap().expect_complete("hel");
    let g = graph.graph();
    let pats: HashSet<_> = hel.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![he, l],
    ]);
    let pats: HashSet<_> = hell.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![hel, l],
    ]);
    drop(g);
    let held = graph.find_sequence("held".chars()).unwrap().expect_complete("held");
    let g = graph.graph();
    let pats: HashSet<_> = held.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![hel, d],
        vec![he, ld],
    ]);
    let pats: HashSet<_> = heldld.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![held, ld],
    ]);
}

#[test]
fn read_sequence1() {
    let mut graph = HypergraphRef::default();
    let ind_hypergraph = graph.read_sequence("hypergraph".chars()).unwrap();
    let gr = graph.graph();
    let h = gr.expect_token_child('h');
    let y = gr.expect_token_child('y');
    let p = gr.expect_token_child('p');
    let e = gr.expect_token_child('e');
    let r = gr.expect_token_child('r');
    let g = gr.expect_token_child('g');
    let a = gr.expect_token_child('a');
    drop(gr);
    {
        let gr = graph.graph();
        let pats: HashSet<_> = ind_hypergraph.vertex(&gr).get_child_pattern_set().into_iter().collect();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![h, y, p, e, r, g, r, a, p, h],
        ]);
        let pid = *ind_hypergraph.vertex(&gr).get_child_patterns().into_iter().next().unwrap().0;
        assert_child_of_at(&gr, h, ind_hypergraph,
            [
                PatternIndex::new(pid, 0),
                PatternIndex::new(pid, 9),
            ]);
        assert_child_of_at(&gr, y, ind_hypergraph,
            [
                PatternIndex::new(pid, 1),
            ]);
        assert_child_of_at(&gr, p, ind_hypergraph,
            [
                PatternIndex::new(pid, 2),
                PatternIndex::new(pid, 8),
            ]);
        assert_child_of_at(&gr, e, ind_hypergraph,
            [
                PatternIndex::new(pid, 3),
            ]);
        assert_child_of_at(&gr, r, ind_hypergraph,
            [
                PatternIndex::new(pid, 6),
                PatternIndex::new(pid, 4),
            ]);
        assert_child_of_at(&gr, a, ind_hypergraph,
            [
                PatternIndex::new(pid, 7),
            ]);
        assert_eq!(ind_hypergraph.width(), 10);
    }
    let hyper = graph.read_sequence("hyper".chars()).unwrap();
    {
        let graph = graph.graph();
        let pats: HashSet<_> = hyper.vertex(&graph).get_child_pattern_set().into_iter().collect();
        assert_eq!(pats, hashset![
            vec![h, y, p, e, r],
        ]);
        assert_eq!(hyper.width(), 5);
        let pats: HashSet<_> = ind_hypergraph.vertex(&graph).get_child_pattern_set().into_iter().collect();
        assert_eq!(pats, hashset![
            vec![hyper, g, r, a, p, h],
        ]);
        assert_eq!(ind_hypergraph.width(), 10);
    }
    let ind_graph = graph.read_sequence("graph".chars()).unwrap();
    let graph = graph.graph();
    let pats: HashSet<_> = ind_graph.vertex(&graph).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![g, r, a, p, h],
    ]);
    assert_eq!(ind_graph.width(), 5);
    let pats: HashSet<_> = ind_hypergraph.vertex(&graph).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![hyper, ind_graph],
    ]);
    assert_eq!(ind_hypergraph.width(), 10);
}

#[test]
fn read_sequence2() {
    let mut graph = HypergraphRef::default();
    let ind_abab = graph.read_sequence("abab".chars()).unwrap();
    let gr = graph.graph();
    let a = gr.expect_token_child('a');
    let b = gr.expect_token_child('b');
    drop(gr);
    let ab = graph.find_ancestor([a, b]).unwrap().expect_complete("ab");
    {
        let gr = graph.graph();
        let pats: HashSet<_> = ind_abab.vertex(&gr).get_child_pattern_set().into_iter().collect();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![ab, ab],
        ]);
        let pid = *ab.vertex(&gr).get_child_patterns().into_iter().next().unwrap().0;
        assert_child_of_at(&gr, a, ab,
            [
                PatternIndex::new(pid, 0),
            ]);
        assert_child_of_at(&gr, b, ab,
            [
                PatternIndex::new(pid, 1),
            ]);
        let pid = *ind_abab.vertex(&gr).get_child_patterns().into_iter().next().unwrap().0;
        assert_child_of_at(&gr, ab, ind_abab,
            [
                PatternIndex::new(pid, 0),
                PatternIndex::new(pid, 1),
            ]);
    }
    let ind_a = graph.read_sequence("a".chars()).unwrap();
    assert_eq!(ab.width(), 2);
    assert_eq!(ind_abab.width(), 4);
    assert_eq!(ind_a, a);
}

#[test]
fn read_infix1() {
    let mut graph = HypergraphRef::default();
    let subdivision = graph.read_sequence("subdivision".chars()).unwrap();
    assert_eq!(subdivision.width(), 11);
    let s = graph.graph().expect_token_child('s');
    let u = graph.graph().expect_token_child('u');
    let b = graph.graph().expect_token_child('b');
    let d = graph.graph().expect_token_child('d');
    let i = graph.graph().expect_token_child('i');
    let v = graph.graph().expect_token_child('v');
    let o = graph.graph().expect_token_child('o');
    let n = graph.graph().expect_token_child('n');
    {
        let graph = graph.graph();
        let pats: HashSet<_> = subdivision.vertex(&graph).get_child_pattern_set().into_iter().collect();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![s, u, b, d, i, v, i, s, i, o, n],
        ]);
        let pid = *subdivision.vertex(&graph).get_child_patterns().into_iter().next().unwrap().0;
        assert_child_of_at(&graph, s, subdivision,
            [
                PatternIndex::new(pid, 0),
                PatternIndex::new(pid, 7),
            ]);
        assert_child_of_at(&graph, u, subdivision,
            [
                PatternIndex::new(pid, 1),
            ]);
        assert_child_of_at(&graph, b, subdivision,
            [
                PatternIndex::new(pid, 2),
            ]);
        assert_child_of_at(&graph, d, subdivision,
            [
                PatternIndex::new(pid, 3),
            ]);
        assert_child_of_at(&graph, i, subdivision,
            [
                PatternIndex::new(pid, 4),
                PatternIndex::new(pid, 6),
                PatternIndex::new(pid, 8),
            ]);
        assert_child_of_at(&graph, v, subdivision,
            [
                PatternIndex::new(pid, 5),
            ]);
        assert_child_of_at(&graph, o, subdivision,
            [
                PatternIndex::new(pid, 9),
            ]);
        assert_child_of_at(&graph, n, subdivision,
            [
                PatternIndex::new(pid, 10),
            ]);
    }
    let visualization = graph.read_sequence("visualization".chars()).unwrap();
    {
        let g = graph.graph();
        let a = g.expect_token_child('a');
        let l = g.expect_token_child('l');
        let z = g.expect_token_child('z');
        let t = g.expect_token_child('t');
        drop(g);

        let vis = graph.find_sequence("vis".chars()).unwrap().expect_complete("vis");
        let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
        let g = graph.graph();
        let pats: HashSet<_> = su.vertex(&g).get_child_pattern_set().into_iter().collect();
        assert_eq!(pats, hashset![
            vec![s, u],
        ]);
        drop(g);
        let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
        let g = graph.graph();
        let pats: HashSet<_> = vis.vertex(&g).get_child_pattern_set().into_iter().collect();
        assert_eq!(pats, hashset![
            vec![vi, s],
        ]);
        drop(g);


        let visu = graph.find_sequence("visu".chars()).unwrap().expect_complete("visu");
        assert!(visu.width() == 4);
        let g = graph.graph();
        let pats: HashSet<_> = visu.vertex(&g).get_child_pattern_set().into_iter().collect();
        assert_eq!(pats, hashset![
            vec![vis, u],
            vec![vi, su],
        ]);
        drop(g);

        let ion = graph.find_sequence("ion".chars()).unwrap().expect_complete("ion");
        let g = graph.graph();
        let pats: HashSet<_> = visualization.vertex(&g).get_child_pattern_set().into_iter().collect();
        assert_eq!(pats, hashset![
            vec![visu, a, l, i, z, a, t, ion],
        ]);
        let pats: HashSet<_> = subdivision.vertex(&g).get_child_pattern_set().into_iter().collect();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![su, b, d, i, vis, ion],
        ]);
    }
    assert_eq!(visualization.width(), 13);
}

#[test]
fn read_infix2() {
    let mut graph = HypergraphRef::default();
    let subvisu = graph.read_sequence("subvisu".chars()).unwrap();
    assert_eq!(subvisu.width(), 7);
    let g = graph.graph();
    let s = g.expect_token_child('s');
    let u = g.expect_token_child('u');
    let b = g.expect_token_child('b');
    let v = g.expect_token_child('v');
    let i = g.expect_token_child('i');
    drop(g);

    let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
    let g = graph.graph();
    let pats: HashSet<_> = su.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![s, u],
    ]);
    let pats: HashSet<_> = subvisu.vertex(&g).get_child_pattern_set().into_iter().collect();
    //println!("{:#?}", );
    assert_eq!(pats, hashset![
        vec![su, b, v, i, su],
    ]);
    drop(g);

    let visub = graph.read_sequence("visub".chars()).unwrap();
    //let visub_patterns = visub.expect_child_patterns(&graph);
    //println!("{:#?}", graph.graph().pattern_strings(visub_patterns.values()));
    assert_eq!(visub.width(), 5);
    let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
    let sub = graph.find_sequence("sub".chars()).unwrap().expect_complete("sub");
    let g = graph.graph();
    let pats: HashSet<_> = sub.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![su, b],
    ]);
    drop(g);

    let visu = graph.find_sequence("visu".chars()).unwrap().expect_complete("visu");
    let g = graph.graph();
    let pats: HashSet<_> = visu.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![vi, su],
    ]);
    let pats: HashSet<_> = visub.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![visu, b],
        vec![vi, sub],
    ]);
    let pats: HashSet<_> = subvisu.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![sub, visu],
    ]);
}

#[test]
fn read_loose_sequence1() {
    let mut graph = HypergraphRef::default();
    let abxaxxb = graph.read_sequence("abxaxxb".chars()).unwrap();
    assert_eq!(abxaxxb.width(), 7);
    let g = graph.graph();
    let a = g.expect_token_child('a');
    let b = g.expect_token_child('b');
    let x = g.expect_token_child('x');

    let pats: HashSet<_> = abxaxxb.vertex(&g).get_child_pattern_set().into_iter().collect();
    //println!("{:#?}", );
    assert_eq!(pats, hashset![
        vec![a, b, x, a, x, x, b],
    ]);
}

#[test]
fn read_repeating_known1() {
    let mut graph = HypergraphRef::default();
    let xyyxy = graph.read_sequence("xyyxy".chars()).unwrap();
    assert_eq!(xyyxy.width(), 5);
    let g = graph.graph();
    let x = g.expect_token_child('x');
    let y = g.expect_token_child('y');

    drop(g);
    let xy = graph.find_sequence("xy".chars()).unwrap().expect_complete("xy");
    let g = graph.graph();

    let pats: HashSet<_> = xy.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![x, y],
    ]);
    let pats: HashSet<_> = xyyxy.vertex(&g).get_child_pattern_set().into_iter().collect();
    //println!("{:#?}", );
    assert_eq!(pats, hashset![
        vec![xy, y, xy],
    ]);
}

#[test]
fn read_multiple_overlaps1() {
    let mut graph = HypergraphRef::default();
    let abcde = graph.read_sequence("abcde".chars()).unwrap();
    // abcde
    //  bcde
    //  bcdea

    let g = graph.graph();
    let a = g.expect_token_child('a');
    let b = g.expect_token_child('b');
    let c = g.expect_token_child('c');
    let d = g.expect_token_child('d');
    let e = g.expect_token_child('e');
    assert_eq!(
        abcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![a, b, c, d, e],
        ]
    );
    drop(g);
    let bcdea = graph.read_sequence("bcdea".chars()).unwrap();
    let bcde = graph.find_sequence("bcde".chars()).unwrap().expect_complete("bcde");
    let g = graph.graph();
    assert_eq!(
        bcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![b, c, d, e],
        ]
    );
    assert_eq!(
        bcdea.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![bcde, a],
        ]
    );
    assert_eq!(
        abcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![a, bcde],
        ]
    );
    drop(g);

    let cdeab = graph.read_sequence("cdeab".chars()).unwrap();
    // abcde
    //  bcde
    //  bcdea
    //   cdea
    //   cdeab
    let cde = graph.find_sequence("cde".chars()).unwrap().expect_complete("cde");
    let ab = graph.find_sequence("ab".chars()).unwrap().expect_complete("ab");
    let cdea = graph.find_sequence("cdea".chars()).unwrap().expect_complete("cdea");

    let g = graph.graph();
    assert_eq!(
        ab.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![a, b],
        ]
    );
    assert_eq!(
        cde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![c, d, e],
        ]
    );
    assert_eq!(
        cdea.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![cde, a],
        ]
    );
    assert_eq!(
        bcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![b, cde],
        ]
    );
    assert_eq!(
        abcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![a, bcde],
            vec![ab, cde],
        ]
    );
    assert_eq!(
        bcdea.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![bcde, a],
            vec![b, cdea],
        ]
    );
    assert_eq!(
        cdeab.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![cde, ab],
            vec![cdea, b],
        ]
    );
    drop(g);
    let deabc = graph.read_sequence("deabc".chars()).unwrap();

    // abcde
    //  bcde
    // a
    //  bcdea
    //   cdea
    //   cdeab
    // ab
    //   cde
    //   cdea
    //    deab
    //    deabc
    // abc
    //    de
    //    dea
    let de = graph.find_sequence("de".chars()).unwrap().expect_complete("de");

    let g = graph.graph();
    assert_eq!(
        de.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![d, e],
        ]
    );
    assert_eq!(
        cde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![c, de],
        ]
    );
    let dea = graph.find_sequence("dea".chars()).unwrap().expect_complete("dea");
    assert_eq!(
        dea.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![de, a],
        ]
    );
    assert_eq!(
        cdea.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![cde, a],
            vec![c, dea],
        ]
    );
    let bc = graph.find_sequence("bc".chars()).unwrap().expect_complete("bc");
    assert_eq!(
        bc.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![b, c],
        ]
    );
    assert_eq!(
        bcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![b, cde],
            vec![bc, de],
        ]
    );
    assert_eq!(
        bcdea.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![bcde, a],
            vec![b, cdea],
            vec![bc, dea],
        ]
    );
    let deab = graph.find_sequence("deab".chars()).unwrap().expect_complete("deab");
    assert_eq!(
        deab.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![de, ab],
            vec![dea, b],
        ]
    );
    let abc = graph.find_sequence("abc".chars()).unwrap().expect_complete("abc");
    assert_eq!(
        abc.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![ab, c],
            vec![a, bc],
        ]
    );
    assert_eq!(
        abcde.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![abc, de],
            vec![a, bcde],
            vec![ab, cde],
        ]
    );
    assert_eq!(
        deabc.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![de, abc],
            vec![dea, bc],
            vec![deab, c],
        ]
    );
    drop(g);
    let eabcd = graph.read_sequence("eabcd".chars()).unwrap();

    let abcd = graph.find_sequence("abcd".chars()).unwrap().expect_complete("abcd");
    let bcd = graph.find_sequence("bcd".chars()).unwrap().expect_complete("bcd");
    let bc = graph.find_sequence("bc".chars()).unwrap().expect_complete("bc");
    let cd = graph.find_sequence("cd".chars()).unwrap().expect_complete("cd");

    let g = graph.graph();

    assert_eq!(
        cd.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![c, d],
        ]
    );
    assert_eq!(
        bcd.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![b, cd],
            vec![bc, d],
        ]
    );
    assert_eq!(
        abcd.vertex(&g).get_child_pattern_set().into_iter().collect::<HashSet<_>>(),
        hashset![
            vec![abc, d],
            vec![a, bcd],
        ]
    );
    drop(g);
    let abcdeabcde = graph.read_sequence("abcdeabcde".chars()).unwrap();
    let g = graph.graph();
    let pats: HashSet<_> = abcdeabcde.vertex(&g).get_child_pattern_set().into_iter().collect();
    assert_eq!(pats, hashset![
        vec![abcde, abcde],
        vec![a,bcdea, bcde],
        vec![ab, cdeab, cde],
        vec![abc, deabc, de],
        vec![abcd, eabcd, e],
    ]);

    //assert_eq!(abcdeabcde.width(), 10);
}