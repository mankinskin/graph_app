use std::iter::{Extend, FusedIterator};

use crate::{TraversalIterator, Tokenize, Traversable, MatchDirection, DirectedTraversalPolicy, FolderNode, TraversalQuery};

#[derive(Clone)]
pub(crate) struct Dft<'a: 'g, 'g, T, Trav, D, Q, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    stack: Vec<(usize, FolderNode<'a, 'g, T, D, Q, S>)>,
    last: (usize, FolderNode<'a, 'g, T, D, Q, S>),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D, Q, S)>
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> Dft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    #[inline]
    pub fn new(trav: &'a Trav, root: FolderNode<'a, 'g, T, D, Q, S>) -> Self {
        Self {
            stack: vec![],
            last: (0, root),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> Iterator for Dft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    type Item = (usize, FolderNode<'a, 'g, T, D, Q, S>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = &self.last;
        self.stack.extend(
            <Self as TraversalIterator<'a, 'g, T, Trav, D, Q, S>>::iter_children(&self.trav, last_node)
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

impl<'a: 'g, 'g, T, Trav, D, Q, S> FusedIterator for Dft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
}

impl<'a: 'g, 'g, T, Trav, D, Q, S> TraversalIterator<'a, 'g, T, Trav, D, Q, S> for Dft<'a, 'g, T, Trav, D, Q, S>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: FolderNode<'a, 'g, T, D, Q, S>) -> Self {
        Dft::new(trav, root)
    }
}