use std::collections::VecDeque;
use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::{Extend, FusedIterator};

use super::*;

#[derive(Clone)]
pub(crate) struct Bft<'a: 'g, 'g, T, Trav, D, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    queue: VecDeque<(usize, FolderNode<'a, 'g, T, D, Q, S>)>,
    last: (usize, FolderNode<'a, 'g, T, D, Q, S>),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D, S)>
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> Bft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    pub fn new(trav: &'a Trav, root: FolderNode<'a, 'g, T, D, Q, S>) -> Self {
        Self {
            queue: VecDeque::new(),
            last: (0, root),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> Iterator for Bft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    type Item = (usize, FolderNode<'a, 'g, T, D, Q, S>);

    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = &self.last;
        self.queue.extend(
            <Self as TraversalIterator<T, Trav, D, Q, S>>::iter_children(self.trav, last_node)
                .into_iter()
                .map(|child| (last_depth + 1, child))
        );
        if let Some((depth, node)) = self.queue.pop_front() {
            self.last = (depth, node.clone());
            Some((depth, node))
        } else {
            None
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> FusedIterator for Bft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> TraversalIterator<'a, 'g, T, Trav, D, Q, S> for Bft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: FolderNode<'a, 'g, T, D, Q, S>) -> Self {
        Bft::new(trav, root)
    }
}