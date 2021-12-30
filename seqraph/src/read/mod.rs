use super::VertexIndex;
use crate::{
    direction::*,
    vertex::*,
    Hypergraph,
};
use std::borrow::Borrow;
mod reader;
pub use reader::*;
//mod async_reader;
//pub use async_reader::*;
#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub(crate) enum NewTokenIndex {
    New(VertexIndex),
    Known(VertexIndex),
}
impl NewTokenIndex {
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }
}
impl Wide for NewTokenIndex {
    fn width(&self) -> usize {
        1
    }
}
impl Indexed for NewTokenIndex {
    fn index(&self) -> &VertexIndex {
        match self {
            Self::New(i) => i,
            Self::Known(i) => i,
        }
    }
}
impl Borrow<VertexIndex> for &'_ NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        (*self).index()
    }
}
impl Borrow<VertexIndex> for &'_ mut NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        (*self).index()
    }
}
pub(crate) type NewTokenIndices = Vec<NewTokenIndex>;
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
    use maplit::hashset;
    use std::collections::HashSet;
    use pretty_assertions::assert_eq;
    #[test]
    fn sync_read_text() {
        let text = "Heldldo world!";
        let mut g: Hypergraph<char> = Hypergraph::default();
        let result = g.read_sequence(text.chars());
        let cap_h = g.expect_token_child('H');
        let e = g.expect_token_child('e');
        let l = g.expect_token_child('l');
        let d = g.expect_token_child('d');
        let o = g.expect_token_child('o');
        let space = g.expect_token_child(' ');
        let w = g.expect_token_child('w');
        let r = g.expect_token_child('r');
        let exclam = g.expect_token_child('!');
        let ld = g.find_sequence("ld".chars()).unwrap().index;
        let res_pats: HashSet<_> = result.vertex(&g).get_child_patterns().into_iter().collect();
        let res_exp = hashset![
            vec![cap_h, e, ld, ld, o, space, w, o, r, ld, exclam],
        ];
        assert_eq!(res_pats, res_exp);
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
