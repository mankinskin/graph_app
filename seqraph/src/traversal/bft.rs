use std::collections::VecDeque;
use std::iter::{Extend, FusedIterator};

use crate::{TraversalIterator, Tokenize, Traversable, MatchDirection, DirectedTraversalPolicy, TraversalFolder};

#[derive(Clone)]
pub(crate) struct Bft<'a: 'g, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    queue: VecDeque<(usize, <S::Folder as TraversalFolder<'a, 'g, T, D>>::Node)>,
    last: (usize, <S::Folder as TraversalFolder<'a, 'g, T, D>>::Node),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D, S)>
}

impl<'a: 'g, 'g, T, Trav, D, S> Bft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    #[inline]
    pub fn new(trav: &'a Trav, root: <S::Folder as TraversalFolder<'a, 'g, T, D>>::Node) -> Self {
        Self {
            queue: VecDeque::new(),
            last: (0, root),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, S> Iterator for Bft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    type Item = (usize, <S::Folder as TraversalFolder<'a, 'g, T, D>>::Node);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = &self.last;
        self.queue.extend(
            <Self as TraversalIterator<T, Trav, D, S>>::iter_children(&self.trav, last_node)
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

impl<'a: 'g, 'g, T, Trav, D, S> FusedIterator for Bft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, Trav, D, S> TraversalIterator<'a, 'g, T, Trav, D, S> for Bft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: <S::Folder as TraversalFolder<'a, 'g, T, D>>::Node) -> Self {
        Bft::new(trav, root)
    }
}