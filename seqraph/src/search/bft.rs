use std::{
    collections::VecDeque,
    iter::{
        Extend,
        FusedIterator,
    },
};

#[derive(Clone)]
pub struct Bft<T, F, I>
    where
        T: Sized,
        F: FnMut(&T) -> I,
        I: Iterator<Item=T>,
{
    queue: VecDeque<(usize, T)>,
    iter_children: F,
}

impl<T, F, I> Bft<T, F, I>
    where
        T: Sized,
        F: FnMut(&T) -> I,
        I: Iterator<Item=T>,
{
    #[inline]
    pub fn new(
        root: T,
        iter_children: F,
    ) -> Self {
        Self {
            queue: VecDeque::from(vec![(0, root)]),
            iter_children,
        }
    }
}

impl<T, F, I> Iterator for Bft<T, F, I>
    where
        T: Sized,
        F: FnMut(&T) -> I,
        I: Iterator<Item=T>,
{
    type Item = (usize, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((depth, node)) = self.queue.pop_front() {
            let children = (self.iter_children)(&node);
            self.queue.extend(children.map(|child| (depth + 1, child)));

            Some((depth, node))
        } else {
            None
        }
    }
}

impl<T, F, I> FusedIterator for Bft<T, F, I>
    where
        T: Sized,
        F: FnMut(&T) -> I,
        I: Iterator<Item=T>,
{}

pub(crate) trait Traversable {
    type Node;
    type State;
}

pub(crate) trait BreadthFirstTraversal<'g> {
    type Trav: Traversable;
    fn end_op(state: <Self::Trav as Traversable>::State) -> Vec<<Self::Trav as Traversable>::Node>;
}
