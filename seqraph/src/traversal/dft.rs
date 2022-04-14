use std::iter::{Extend, FusedIterator};

use crate::{TraversalNode, TraversalIterator, Tokenize, Traversable, MatchDirection, DirectedTraversalPolicy};

#[derive(Clone)]
pub(crate) struct Dft<'a: 'g, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    stack: Vec<(usize, TraversalNode)>,
    last: (usize, TraversalNode),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D, S)>
}

impl<'a: 'g, 'g, T, Trav, D, S> Dft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    #[inline]
    pub fn new(trav: &'a Trav, root: TraversalNode) -> Self {
        Self {
            stack: vec![],
            last: (0, root),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, S> Iterator for Dft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    type Item = (usize, TraversalNode);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = &self.last;
        self.stack.extend(
            <Self as TraversalIterator<'a, 'g, T, Trav, D, S>>::iter_children(&self.trav, last_node)
                .into_iter()
                .rev()
                .map(|child| (last_depth + 1, child))
        );
        if let Some((depth, node)) = self.stack.pop() {
            self.last = (depth, node.clone());
            Some((depth, node))
        } else {
            None
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, S> FusedIterator for Dft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, Trav, D, S> TraversalIterator<'a, 'g, T, Trav, D, S> for Dft<'a, 'g, T, Trav, D, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: TraversalNode) -> Self {
        Dft::new(trav, root)
    }
}