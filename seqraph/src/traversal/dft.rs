use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::{Extend, FusedIterator};
use super::*;


#[derive(Clone)]
pub(crate) struct Dft<'a: 'g, 'g, T, D, Trav, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    stack: Vec<(usize, TraversalNode<Q>)>,
    last: (usize, TraversalNode<Q>),
    cache: TraversalCache<Q>,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D, Q, S)>
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> Dft<'a, 'g, T, D, Trav, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    pub fn new(trav: &'a Trav, root: TraversalNode<Q>) -> Self {
        Self {
            stack: vec![],
            last: (0, root),
            cache: Default::default(),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'a: 'g, 'g, T, D, Trav, Q, S> Iterator for Dft<'a, 'g, T, D, Trav, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    type Item = (usize, TraversalNode<Q>);

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

impl<'a: 'g, 'g, T, Trav, D, Q, S> FusedIterator for Dft<'a, 'g, T, D, Trav, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> TraversalIterator<'a, 'g, T, D, Trav, Q, S> for Dft<'a, 'g, T, D, Trav, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: TraversalNode<Q>) -> Self {
        Dft::new(trav, root)
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
    fn cache_mut(&mut self) -> &mut TraversalCache<Q> {
        &mut self.cache
    }
    fn extend_nodes(&mut self, next_nodes: impl DoubleEndedIterator<Item=(usize, TraversalNode<Q>)>) {
        self.stack.extend(next_nodes.rev());
    }
}