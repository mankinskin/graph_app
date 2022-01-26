use crate::{
    direction::*,
    vertex::*,
    Hypergraph,
};
mod reader;
pub use reader::*;
//mod async_reader;
//pub use async_reader::*;

impl<T: Tokenize + Send + std::fmt::Display> Hypergraph<T> {
    pub fn right_reader(&mut self) -> Reader<'_, T, Right> {
        Reader::new(self)
    }
    pub fn left_reader(&mut self) -> Reader<'_, T, Left> {
        Reader::new(self)
    }
    pub fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> Child {
        self.right_reader().read_sequence(sequence)
    }
}

#[cfg(test)]
mod tests {
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
        let mut g: Hypergraph<char> = Hypergraph::default();
        let result = g.read_sequence("heldldo world!".chars());
        let h = g.expect_token_child('h');
        let e = g.expect_token_child('e');
        let l = g.expect_token_child('l');
        let d = g.expect_token_child('d');
        let o = g.expect_token_child('o');
        let space = g.expect_token_child(' ');
        let w = g.expect_token_child('w');
        let r = g.expect_token_child('r');
        let exclam = g.expect_token_child('!');
        let ld = g.find_sequence("ld".chars()).unwrap().root;
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
        let mut g: Hypergraph<char> = Hypergraph::default();
        let heldld = g.read_sequence("heldld".chars());
        let h = g.expect_token_child('h');
        let e = g.expect_token_child('e');
        let l = g.expect_token_child('l');
        let d = g.expect_token_child('d');
        let ld = g.find_sequence("ld".chars()).unwrap().expect_complete("ld");
        let pats: HashSet<_> = ld.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![l, d],
        ]);
        let pats: HashSet<_> = heldld.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![h, e, ld, ld],
        ]);
        let hell = g.read_sequence("hell".chars());
        let he = g.find_sequence("he".chars()).unwrap().expect_complete("he");
        let pats: HashSet<_> = he.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![h, e],
        ]);
        let hel = g.find_sequence("hel".chars()).unwrap().expect_complete("hel");
        let pats: HashSet<_> = hel.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![he, l],
        ]);
        let pats: HashSet<_> = hell.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![hel, l],
        ]);
        let dld = g.find_sequence("dld".chars()).unwrap().expect_complete("dld");
        let pats: HashSet<_> = dld.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![d, ld],
        ]);
        let pats: HashSet<_> = heldld.vertex(&g).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![hel, dld],
            vec![he, ld, ld],
        ]);
    }
    #[test]
    fn read_prefix_postfix1() {
        let mut graph = Hypergraph::default();
        let ind_hypergraph = graph.read_sequence("hypergraph".chars());
        let h = graph.expect_token_child('h');
        let y = graph.expect_token_child('y');
        let p = graph.expect_token_child('p');
        let e = graph.expect_token_child('e');
        let r = graph.expect_token_child('r');
        let g = graph.expect_token_child('g');
        let a = graph.expect_token_child('a');
        let pats = ind_hypergraph.vertex(&graph).get_child_pattern_set();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![h, y, p, e, r, g, r, a, p, h],
        ]);
        let pid = *ind_hypergraph.vertex(&graph).get_children().into_iter().next().unwrap().0;
        assert_child_of_at(&graph, h, ind_hypergraph,
            [
                PatternIndex::new(pid, 0),
                PatternIndex::new(pid, 9),
            ]);
        assert_child_of_at(&graph, y, ind_hypergraph,
            [
                PatternIndex::new(pid, 1),
            ]);
        assert_child_of_at(&graph, p, ind_hypergraph,
            [
                PatternIndex::new(pid, 2),
                PatternIndex::new(pid, 8),
            ]);
        assert_child_of_at(&graph, e, ind_hypergraph,
            [
                PatternIndex::new(pid, 3),
            ]);
        assert_child_of_at(&graph, r, ind_hypergraph,
            [
                PatternIndex::new(pid, 6),
                PatternIndex::new(pid, 4),
            ]);
        assert_child_of_at(&graph, a, ind_hypergraph,
            [
                PatternIndex::new(pid, 7),
            ]);
        assert_eq!(ind_hypergraph.width(), 10);
        let hyper = graph.read_sequence("hyper".chars());
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
        let ind_graph = graph.read_sequence("graph".chars());
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
        let mut graph = Hypergraph::default();
        let subdivision = graph.read_sequence("subdivision".chars());
        assert_eq!(subdivision.width(), 11);
        let s = graph.expect_token_child('s');
        let u = graph.expect_token_child('u');
        let b = graph.expect_token_child('b');
        let d = graph.expect_token_child('d');
        let i = graph.expect_token_child('i');
        let v = graph.expect_token_child('v');
        let o = graph.expect_token_child('o');
        let n = graph.expect_token_child('n');
        let pats = subdivision.vertex(&graph).get_child_pattern_set();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![s, u, b, d, i, v, i, s, i, o, n],
        ]);
        let pid = *subdivision.vertex(&graph).get_children().into_iter().next().unwrap().0;
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
        assert_eq!(subdivision.width(), 11);
        let visualization = graph.read_sequence("visualization".chars());
        let a = graph.expect_token_child('a');
        let l = graph.expect_token_child('l');
        let z = graph.expect_token_child('z');
        let t = graph.expect_token_child('t');
        let vis = graph.find_sequence("vis".chars()).unwrap().expect_complete("vis");
        let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
        let pats = vis.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![vi, s],
        ]);
        let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
        let pats = su.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![s, u],
        ]);
        let visu = graph.find_sequence("visu".chars()).unwrap().expect_complete("visu");
        let pats = visu.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![vis, u],
            vec![vi, su],
        ]);
        let ion = graph.find_sequence("ion".chars()).unwrap().expect_complete("ion");
        let pats = visualization.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![visu, a, l, i, z, a, t, ion],
        ]);
        let pats = subdivision.vertex(&graph).get_child_pattern_set();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![su, b, d, i, vis, ion],
        ]);
    }
    #[test]
    fn read_infix2() {
        let mut graph = Hypergraph::default();
        let subvisu = graph.read_sequence("subvisu".chars());
        assert_eq!(subvisu.width(), 7);
        let s = graph.expect_token_child('s');
        let u = graph.expect_token_child('u');
        let b = graph.expect_token_child('b');
        let v = graph.expect_token_child('v');
        let i = graph.expect_token_child('i');
        let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
        let pats = su.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![s, u],
        ]);
        let pats = subvisu.vertex(&graph).get_child_pattern_set();
        //println!("{:#?}", );
        assert_eq!(pats, hashset![
            vec![su, b, v, i, su],
        ]);
        let visub = graph.read_sequence("visub".chars());
        assert_eq!(visub.width(), 5);
        //println!("{:#?}", );
        let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
        let sub = graph.find_sequence("sub".chars()).unwrap().expect_complete("sub");
        let pats = sub.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![su, b],
        ]);
        let visu = graph.find_sequence("visu".chars()).unwrap().expect_complete("visu");
        let pats = visu.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![vi, su],
        ]);
        let pats = visub.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![visu, b],
            vec![vi, sub],
        ]);
        let pats = subvisu.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![sub, visu],
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
}
