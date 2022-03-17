use std::collections::VecDeque;
use std::iter::{Extend, FusedIterator};

use crate::{TraversalIterator, TraversalNode, Tokenize, Traversable, MatchDirection, DirectedTraversalPolicy};

#[derive(Clone)]
pub(crate) struct Bft<'g, T, Trav, D, S>
where
    T: Tokenize,
    Trav: Traversable<T> + 'g,
    D: MatchDirection,
    S: DirectedTraversalPolicy<T, D, Trav=Trav>,
{
    queue: VecDeque<(usize, TraversalNode)>,
    trav: &'g Trav,
    _ty: std::marker::PhantomData<(T, D, S)>
}

impl<'g, T, Trav, D, S> Bft<'g, T, Trav, D, S>
where
    T: Tokenize,
    Trav: Traversable<T>,
    D: MatchDirection,
    S: DirectedTraversalPolicy<T, D, Trav=Trav>,
{
    #[inline]
    pub fn new(trav: &'g Trav, root: TraversalNode) -> Self {
        Self {
            queue: VecDeque::from(vec![(0, root)]),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'g, T, Trav, D, S> Iterator for Bft<'g, T, Trav, D, S>
where
    T: Tokenize,
    Trav: Traversable<T>,
    D: MatchDirection,
    S: DirectedTraversalPolicy<T, D, Trav=Trav>,
{
    type Item = (usize, TraversalNode);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((depth, node)) = self.queue.pop_front() {
            self.queue.extend(
                <Self as TraversalIterator<T, Trav, D, S>>::iter_children(self.trav, &node)
                .into_iter()
                .map(|child| (depth + 1, child))
            );

            Some((depth, node))
        } else {
            None
        }
    }
}

impl<'g, T, Trav, D, S> FusedIterator for Bft<'g, T, Trav, D, S>
where
    T: Tokenize,
    Trav: Traversable<T>,
    D: MatchDirection,
    S: DirectedTraversalPolicy<T, D, Trav=Trav>,
{
}

impl<'g, T, Trav, D, S> TraversalIterator<'g, T, Trav, D, S> for Bft<'g, T, Trav, D, S>
where
    T: Tokenize,
    Trav: Traversable<T>,
    D: MatchDirection,
    S: DirectedTraversalPolicy<T, D, Trav=Trav>,
{
    fn new(trav: &'g Trav, root: TraversalNode) -> Self {
        Bft::new(trav, root)
    }
}