use crate::*;
//use tokio::sync::mpsc;
//use tokio_stream::wrappers::*;
use maplit::{hashset, hashmap};
use std::collections::HashSet;
use pretty_assertions::assert_eq;

fn assert_child_of_at<T: Tokenize>(graph: &Hypergraph<T>, child: impl AsChild, parent: impl AsChild, pattern_indices: impl IntoIterator<Item=PatternIndex>) {
    assert_eq!(*graph.expect_parents(child), hashmap![
        parent.index() => Parent {
            pattern_indices: pattern_indices.into_iter().collect(),
            width: parent.width(),
        },
    ]);
}
#[test]
fn sync_read_text1() {
    let mut graph: HypergraphRef<char> = HypergraphRef::from(Hypergraph::default());
    let result = graph.read_sequence("heldldo world!".chars());
    let g = graph.read().unwrap();
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
    let g = graph.read().unwrap();
    let pats: HashSet<_> = ld.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![l, d],
    ]);
    let pats: HashSet<_> = result.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![h, e, ld, ld, o, space, w, o, r, ld, exclam],
    ]);
}
#[test]
fn sync_read_text2() {
    let mut graph = HypergraphRef::default();
    let heldld = graph.read_sequence("heldld".chars());
    let g = graph.read().unwrap();
    let h = g.expect_token_child('h');
    let e = g.expect_token_child('e');
    let l = g.expect_token_child('l');
    let d = g.expect_token_child('d');
    drop(g);
    let ld = graph.find_sequence("ld".chars()).unwrap().expect_complete("ld");
    let g = graph.read().unwrap();
    let pats: HashSet<_> = ld.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![l, d],
    ]);
    let pats: HashSet<_> = heldld.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![h, e, ld, ld],
    ]);
    drop(g);
    let hell = graph.read_sequence("hell".chars());
    let he = graph.find_sequence("he".chars()).unwrap().expect_complete("he");
    let g = graph.read().unwrap();
    let pats: HashSet<_> = he.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![h, e],
    ]);
    drop(g);
    let hel = graph.find_sequence("hel".chars()).unwrap().expect_complete("hel");
    let g = graph.read().unwrap();
    let pats: HashSet<_> = hel.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![he, l],
    ]);
    let pats: HashSet<_> = hell.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![hel, l],
    ]);
    drop(g);
    let held = graph.find_sequence("held".chars()).unwrap().expect_complete("held");
    let g = graph.read().unwrap();
    let pats: HashSet<_> = held.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![hel, d],
        vec![he, ld],
    ]);
    let pats: HashSet<_> = heldld.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![held, ld],
    ]);
}
#[test]
fn read_prefix_postfix1() {
    let mut graph = HypergraphRef::default();
    let ind_hypergraph = graph.read_sequence("hypergraph".chars());
    let gr = graph.read().unwrap();
    let h = gr.expect_token_child('h');
    let y = gr.expect_token_child('y');
    let p = gr.expect_token_child('p');
    let e = gr.expect_token_child('e');
    let r = gr.expect_token_child('r');
    let g = gr.expect_token_child('g');
    let a = gr.expect_token_child('a');
    drop(gr);
    {
        let gr = graph.read().unwrap();
        let pats = ind_hypergraph.vertex(&gr).get_child_pattern_set();
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
    let hyper = graph.read_sequence("hyper".chars());
    {
        let graph = graph.read().unwrap();
        let pats = hyper.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![h, y, p, e, r],
        ]);
        assert_eq!(hyper.width(), 5);
        let pats = ind_hypergraph.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![hyper, g, r, a, p, h],
        ]);
        assert_eq!(ind_hypergraph.width(), 10);
    }
    let ind_graph = graph.read_sequence("graph".chars());
    let graph = graph.read().unwrap();
    let pats = ind_graph.vertex(&graph).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![g, r, a, p, h],
    ]);
    assert_eq!(ind_graph.width(), 5);
    let pats = ind_hypergraph.vertex(&graph).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![hyper, ind_graph],
    ]);
    assert_eq!(ind_hypergraph.width(), 10);
}
#[test]
fn read_infix1() {
    let mut graph = HypergraphRef::default();
    let subdivision = graph.read_sequence("subdivision".chars());
    assert_eq!(subdivision.width(), 11);
    let s = graph.read().unwrap().expect_token_child('s');
    let u = graph.read().unwrap().expect_token_child('u');
    let b = graph.read().unwrap().expect_token_child('b');
    let d = graph.read().unwrap().expect_token_child('d');
    let i = graph.read().unwrap().expect_token_child('i');
    let v = graph.read().unwrap().expect_token_child('v');
    let o = graph.read().unwrap().expect_token_child('o');
    let n = graph.read().unwrap().expect_token_child('n');
    {
        let graph = graph.read().unwrap();
        let pats = subdivision.vertex(&graph).get_child_pattern_set();
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
    let visualization = graph.read_sequence("visualization".chars());
    assert_eq!(visualization.width(), 13);
    {
        let g = graph.read().unwrap();
        let a = g.expect_token_child('a');
        let l = g.expect_token_child('l');
        let z = g.expect_token_child('z');
        let t = g.expect_token_child('t');
        drop(g);

        let vis = graph.find_sequence("vis".chars()).unwrap().expect_complete("vis");
        let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
        let g = graph.read().unwrap();
        let pats = su.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![s, u],
        ]);
        drop(g);
        let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
        let g = graph.read().unwrap();
        let pats = vis.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![vi, s],
        ]);
        drop(g);


        let visu = graph.find_sequence("visu".chars()).unwrap().expect_complete("visu");
        let g = graph.read().unwrap();
        let pats = visu.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![vis, u],
            vec![vi, su],
        ]);
        drop(g);

        let ion = graph.find_sequence("ion".chars()).unwrap().expect_complete("ion");
        let g = graph.read().unwrap();
        let pats = visualization.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![visu, a, l, i, z, a, t, ion],
        ]);
        let pats = subdivision.vertex(&g).get_child_pattern_set();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![su, b, d, i, vis, ion],
        ]);
    }
}
#[test]
fn read_infix2() {
    let mut graph = HypergraphRef::default();
    let subvisu = graph.read_sequence("subvisu".chars());
    assert_eq!(subvisu.width(), 7);
    let g = graph.read().unwrap();
    let s = g.expect_token_child('s');
    let u = g.expect_token_child('u');
    let b = g.expect_token_child('b');
    let v = g.expect_token_child('v');
    let i = g.expect_token_child('i');
    drop(g);

    let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
    let g = graph.read().unwrap();
    let pats = su.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![s, u],
    ]);
    let pats = subvisu.vertex(&g).get_child_pattern_set();
    //println!("{:#?}", );
    assert_eq!(pats, hashset![
        vec![su, b, v, i, su],
    ]);
    drop(g);

    let visub = graph.read_sequence("visub".chars());
    let visub_patterns = visub.expect_child_patterns(&graph);
    assert_eq!(visub.width(), 5);
    println!("{:#?}", graph.read().unwrap().pattern_strings(visub_patterns.values()));
    let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
    let sub = graph.find_sequence("sub".chars()).unwrap().expect_complete("sub");
    let g = graph.read().unwrap();
    let pats = sub.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![su, b],
    ]);
    drop(g);

    let visu = graph.find_sequence("visu".chars()).unwrap().expect_complete("visu");
    let g = graph.read().unwrap();
    let pats = visu.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![vi, su],
    ]);
    let pats = visub.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![visu, b],
        vec![vi, sub],
    ]);
    let pats = subvisu.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![sub, visu],
    ]);
}
#[test]
fn read_loose_sequence1() {
    let mut graph = HypergraphRef::default();
    let abxaxxb = graph.read_sequence("abxaxxb".chars());
    assert_eq!(abxaxxb.width(), 7);
    let g = graph.read().unwrap();
    let a = g.expect_token_child('a');
    let b = g.expect_token_child('b');
    let x = g.expect_token_child('x');

    let pats = abxaxxb.vertex(&g).get_child_pattern_set();
    //println!("{:#?}", );
    assert_eq!(pats, hashset![
        vec![a, b, x, a, x, x, b],
    ]);
}
#[test]
fn read_repeating_known1() {
    let mut graph = HypergraphRef::default();
    let xyyxy = graph.read_sequence("xyyxy".chars());
    assert_eq!(xyyxy.width(), 5);
    let g = graph.read().unwrap();
    let x = g.expect_token_child('x');
    let y = g.expect_token_child('y');

    drop(g);
    let xy = graph.find_sequence("xy".chars()).unwrap().expect_complete("xy");
    let g = graph.read().unwrap();

    let pats = xy.vertex(&g).get_child_pattern_set();
    assert_eq!(pats, hashset![
        vec![x, y],
    ]);
    let pats = xyyxy.vertex(&g).get_child_pattern_set();
    //println!("{:#?}", );
    assert_eq!(pats, hashset![
        vec![xy, y, xy],
    ]);
}
//#[tokio::test]
//async fn async_read_text() {
//    let (mut tx, mut rx) = mpsc::unbounded_channel::<char>();
//    let text = "Hello world!";
//    text.chars().for_each(|c| tx.send(c).unwrap());
//    let mut g = Hypergraph::default();
//    let rx = UnboundedReceiverStream::new(rx);
//    let result = g.read_sequence(text.chars().collect());
//    assert_eq!(result.width, text.len());
//}