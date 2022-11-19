use std::collections::VecDeque;
use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::Extend;

use super::*;

#[derive(Debug)]
pub(crate) struct Bft<'a, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
{
    queue: VecDeque<(usize, TraversalNode<R, Q>)>,
    cache: TraversalCache<R, Q>,
    last: (usize, TraversalNode<R, Q>),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'a T, D, S, R)>
}

impl<'a, T, D, Trav, Q, R, S> Unpin for Bft<'a, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T>,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
{
}

impl<'a, T, D, Trav, Q, R, S> Iterator for Bft<'a, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T>,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
{
    type Item = (usize, TraversalNode<R, Q>);

    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = self.last.clone();
        self.cached_extend(last_depth, last_node);
        if let Some((depth, node)) = self.queue.pop_front() {
            self.last = (depth, node.clone());
            Some((depth, node))
        } else {
            None
        }
    }
}

//impl<T, Trav, D, Q, R, S> FusedIterator for Bft<T, D, Trav, Q, R, S>
//where
//    T: Tokenize,
//    Trav: Traversable<T>,
//    D: MatchDirection,
//    Q: TraversalQuery,
//    R: ResultKind,
//    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
//{
//}

impl<'a, T, Trav, D, Q, S, R> TraversalIterator<'a, T, D, Trav, Q, S, R> for Bft<'a, T, D, Trav, Q, R, S>
where
    T: Tokenize,
    Trav: Traversable<T>,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: TraversalNode<R, Q>) -> Self {
        Self {
            queue: VecDeque::new(),
            last: (0, root),
            cache: Default::default(),
            trav,
            _ty: Default::default(),
        }
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
    fn cache_mut(&mut self) -> &mut TraversalCache<R, Q> {
        &mut self.cache
    }
    fn extend_nodes(&mut self, next_nodes: impl IntoIterator<IntoIter=impl DoubleEndedIterator<Item=(usize, TraversalNode<R, Q>)>>) {
        self.queue.extend(next_nodes.into_iter());
    }
}