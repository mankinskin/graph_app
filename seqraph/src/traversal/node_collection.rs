use std::collections::VecDeque;
use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::Extend;

use super::*;

pub trait NodeCollection<Q, R>: Default
    where
        Q: BaseQuery,
        R: ResultKind,
{
    fn pop_next(&mut self) -> Option<(usize, TraversalNode<R, Q>)>;
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalNode<R, Q>)>,
        T: IntoIterator<Item = (usize, TraversalNode<R, Q>), IntoIter=It>
    >(&mut self, iter: T);
}
pub struct OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        Trav: Traversable<T>,
        D: MatchDirection,
        Q: BaseQuery,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<Q, R>,
{
    collection: O,
    cache: TraversalCache<R, Q>,
    last: (usize, TraversalNode<R, Q>),
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'a T, D, S, R)>
}
impl<'a, T, D, Trav, Q, R, S, O> Unpin for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        Q: BaseQuery,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<Q, R>,
{
}
impl<'a, T, Trav, D, Q, S, R, O> TraversalIterator<'a, T, D, Trav, Q, S, R> for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        Trav: Traversable<T>,
        D: MatchDirection,
        Q: BaseQuery,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<Q, R>,
{
    fn new(trav: &'a Trav, query: Q) -> Option<Self> {
        let index = query.path_child(trav);
        let start = StartNode::new(index, query);
        let cache = TraversalCache::new(&start);
        Some(Self {
            collection: Default::default(),
            last: (0, TraversalNode::Start(start)),
            cache,
            trav,
            _ty: Default::default(),
        })
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
    fn cache_mut(&mut self) -> &mut TraversalCache<R, Q> {
        &mut self.cache
    }
    //fn extend_nodes(&mut self, last_depth: usize, last_node: TraversalNode<R, Q>) {
    //    self.collection.extend(self.next_nodes(last_depth, last_node).into_iter());
    //}
}
impl<'a, T, D, Trav, Q, R, S, O> Iterator for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
where
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T>,
    Q: BaseQuery,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
    O: NodeCollection<Q, R>,
{
    type Item = (usize, TraversalNode<R, Q>);

    fn next(&mut self) -> Option<Self::Item> {
        let (last_depth, last_node) = self.last.clone();
        let (_key, next_nodes) = self.next_nodes(last_node);
        self.collection.extend(
            next_nodes
                .into_iter()
                .map(|node| (last_depth + 1, node))
        );
        if let Some((depth, node)) = self.collection.pop_next() {
            self.last = (depth, node.clone());
            Some((depth, node))
        } else {
            None
        }
    }
}
pub type Bft<'a, T, D, Trav, Q, R, S> = OrderedTraverser<'a, T, D, Trav, Q, R, S, BftQueue<Q, R>>;
#[allow(unused)]
pub type Dft<'a, T, D, Trav, Q, R, S> = OrderedTraverser<'a, T, D, Trav, Q, R, S, DftStack<Q, R>>;

#[derive(Debug)]
pub struct BftQueue<Q, R>
    where
        Q: BaseQuery,
        R: ResultKind,
{
    queue: VecDeque<(usize, TraversalNode<R, Q>)>,
    _ty: std::marker::PhantomData<(Q, R)>
}
impl<Q, R> NodeCollection<Q, R> for BftQueue<Q, R>
    where
        Q: BaseQuery,
        R: ResultKind,
{
    fn pop_next(&mut self) -> Option<(usize, TraversalNode<R, Q>)> {
        self.queue.pop_front()
    }
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalNode<R, Q>)>,
        T: IntoIterator<Item = (usize, TraversalNode<R, Q>), IntoIter=It>
    >(&mut self, iter: T) {
        self.queue.extend(iter)
    }
}
impl<Q, R> Default for BftQueue<Q, R>
    where
        Q: BaseQuery,
        R: ResultKind,
{
    fn default() -> Self {
        Self {
            queue: Default::default(),
            _ty: Default::default(),
        }
    }
}
#[derive(Debug)]
pub struct DftStack<Q, R>
where
    Q: BaseQuery,
    R: ResultKind,
{
    stack: Vec<(usize, TraversalNode<R, Q>)>,
    _ty: std::marker::PhantomData<(Q, R)>
}

impl<Q, R> NodeCollection<Q, R> for DftStack<Q, R>
    where
        Q: BaseQuery,
        R: ResultKind,
{
    fn pop_next(&mut self) -> Option<(usize, TraversalNode<R, Q>)> {
        self.stack.pop()
    }
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalNode<R, Q>)>,
        T: IntoIterator<Item = (usize, TraversalNode<R, Q>), IntoIter=It>
    >(&mut self, iter: T) {
        self.stack.extend(iter.into_iter().rev())
    }
}
impl<Q, R> Default for DftStack<Q, R>
    where
        Q: BaseQuery,
        R: ResultKind,
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
            _ty: Default::default(),
        }
    }
}