use std::collections::VecDeque;
use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::Extend;

use super::*;

#[derive(Debug)]
pub(crate) struct Bft<'a: 'g, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    R: ResultKind + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
    queue: VecDeque<(usize, TraversalNode<R, Q>)>,
    cache: TraversalCache<R, Q>,
    last: (usize, TraversalNode<R, Q>),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, &'a D, S, R)>
}

impl<'a: 'g, 'g, T, D, Trav, Q, R, S> Unpin for Bft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Trav: Traversable<'a, 'g, T>,
    Q: TraversalQuery + 'a,
    R: ResultKind + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, D, Trav, Q, R, S> Stream for Bft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Trav: Traversable<'a, 'g, T>,
    Q: TraversalQuery + 'a,
    R: ResultKind + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
{
    type Item = (usize, TraversalNode<R, Q>);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> futures::task::Poll<Option<Self::Item>> {
        let (last_depth, last_node) = self.last.clone();
        let poll = self.cached_extend(last_depth, last_node).poll_unpin(cx);
        if let Poll::Ready(()) = poll {
            Poll::Ready(
                if let Some((depth, node)) = self.queue.pop_front() {
                    self.last = (depth, node.clone());
                    Some((depth, node))
                } else {
                    None
                }
            )
        } else {
            Poll::Pending
        }
    }
}

//impl<'a: 'g, 'g, T, Trav, D, Q, R, S> FusedIterator for Bft<'a, 'g, T, D, Trav, Q, R, S>
//where
//    T: Tokenize + 'a,
//    Trav: Traversable<'a, 'g, T>,
//    D: MatchDirection + 'a,
//    Q: TraversalQuery + 'a,
//    R: ResultKind + 'a,
//    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
//{
//}

impl<'a: 'g, 'g, T, Trav, D, Q, S, R> TraversalIterator<'a, 'g, T, D, Trav, Q, S, R> for Bft<'a, 'g, T, D, Trav, Q, R, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    R: ResultKind + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
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