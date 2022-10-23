use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::{Extend, FusedIterator};
use super::*;


#[derive(Clone)]
pub(crate) struct Dft<'a: 'g, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
    stack: Vec<(usize, TraversalNode<R, Q>)>,
    last: (usize, TraversalNode<R, Q>),
    cache: TraversalCache<R, Q>,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D, Q, R, S)>
}

impl<'a: 'g, 'g, T, Trav, D, Q, R, S> Dft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
    pub fn new(trav: &'a Trav, root: TraversalNode<R, Q>) -> Self {
        Self {
            stack: vec![],
            last: (0, root),
            cache: Default::default(),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'a: 'g, 'g, T, D, Trav, Q, R, S> Iterator for Dft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
    type Item = (usize, TraversalNode<R, Q>);

    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = self.last.clone();
        self.cached_extend(last_depth, last_node);
        if let Some((depth, node)) = self.stack.pop() {
            self.last = (depth, node.clone());
            Some((depth, node))
        } else {
            None
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, Q, R, S> FusedIterator for Dft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, Trav, D, Q, R, S> TraversalIterator<'a, 'g, T, D, Trav, Q, S, R> for Dft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: TraversalNode<R, Q>) -> Self {
        Dft::new(trav, root)
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
    fn cache_mut(&mut self) -> &mut TraversalCache<R, Q> {
        &mut self.cache
    }
    fn extend_nodes(&mut self, next_nodes: impl IntoIterator<IntoIter=impl DoubleEndedIterator<Item=(usize, TraversalNode<R, Q>)>>) {
        self.stack.extend(next_nodes.into_iter().rev());
    }
}