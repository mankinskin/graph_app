

struct BftTraversal<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath> for BftTraversal<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
}

#[test]
fn traversal1() {
}