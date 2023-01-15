use std::collections::VecDeque;
use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::Extend;

use super::*;

pub trait ExtendStates<R, Q>
    where
        R: ResultKind,
        Q: BaseQuery,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R, Q>)>,
        T: IntoIterator<Item = (usize, TraversalState<R, Q>), IntoIter=It>
    >(&mut self, iter: T);
}
pub trait NodeCollection<R, Q>: From<StartState<R, Q>> + ExtendStates<R, Q>
    where
        R: ResultKind,
        Q: BaseQuery,
{
    fn pop_next(&mut self) -> Option<(usize, TraversalState<R, Q>)>;
}
pub struct OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        Trav: Traversable<T>,
        D: MatchDirection,
        Q: QueryPath,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<R, Q>,
{
    collection: O,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'a T, D, S, R, Trav, Q)>
}
impl<'a, T, D, Trav, Q, R, S, O> Unpin for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        Q: QueryPath,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<R, Q>,
{
}
impl<'a, T, D, Trav, Q, R, S, O> ExtendStates<R, Q> for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        Q: QueryPath,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<R, Q>,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R, Q>)>,
        In: IntoIterator<Item = (usize, TraversalState<R, Q>), IntoIter=It>
    >(&mut self, iter: In) {
        self.collection.extend(iter)
    }
}
impl<'a, T, Trav, D, Q, S, R, O> TraversalIterator<'a, T, D, Trav, Q, S, R> for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
    where
        T: Tokenize,
        Trav: Traversable<T> + 'a + TraversalFolder<T, D, Q, S, R>,
        D: MatchDirection,
        Q: QueryPath,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
        O: NodeCollection<R, Q>,
{
    fn new(trav: &'a Trav, start: StartState<R, Q>) -> Self {
        Self {
            collection: O::from(start),
            trav,
            _ty: Default::default(),
        }
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
}
impl<'a, T, D, Trav, Q, R, S, O> Iterator for OrderedTraverser<'a, T, D, Trav, Q, R, S, O>
where
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T> + TraversalFolder<T, D, Q, S, R>,
    Q: QueryPath,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
    O: NodeCollection<R, Q>,
{
    type Item = (usize, TraversalState<R, Q>);

    fn next(&mut self) -> Option<Self::Item> {
        self.collection.pop_next()
    }
}
pub type Bft<'a, T, D, Trav, Q, R, S> = OrderedTraverser<'a, T, D, Trav, Q, R, S, BftQueue<R, Q>>;
#[allow(unused)]
pub type Dft<'a, T, D, Trav, Q, R, S> = OrderedTraverser<'a, T, D, Trav, Q, R, S, DftStack<R, Q>>;

#[derive(Debug)]
pub struct BftQueue<R, Q>
    where
        R: ResultKind,
        Q: QueryPath,
{
    queue: VecDeque<(usize, TraversalState<R, Q>)>,
    _ty: std::marker::PhantomData<(Q, R)>
}
impl<R: ResultKind, Q: QueryPath> From<StartState<R, Q>> for BftQueue<R, Q> {
    fn from(start: StartState<R, Q>) -> Self {
        Self {
            queue: VecDeque::from([(0, TraversalState::Start(start))]),
            _ty: Default::default(),
        }
    }
}
impl<R, Q> NodeCollection<R, Q> for BftQueue<R, Q>
    where
        Q: QueryPath,
        R: ResultKind,
{
    fn pop_next(&mut self) -> Option<(usize, TraversalState<R, Q>)> {
        self.queue.pop_front()
    }
}
impl<R, Q> ExtendStates<R, Q> for BftQueue<R, Q>
    where
        Q: QueryPath,
        R: ResultKind,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R, Q>)>,
        T: IntoIterator<Item = (usize, TraversalState<R, Q>), IntoIter=It>
    >(&mut self, iter: T) {
        self.queue.extend(iter)
    }
}
impl<R, Q> Default for BftQueue<R, Q>
    where
        Q: QueryPath,
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
pub struct DftStack<R, Q>
where
    Q: QueryPath,
    R: ResultKind,
{
    stack: Vec<(usize, TraversalState<R, Q>)>,
    _ty: std::marker::PhantomData<(Q, R)>
}
impl<R: ResultKind, Q: QueryPath> From<StartState<R, Q>> for DftStack<R, Q> {
    fn from(start: StartState<R, Q>) -> Self {
        Self {
            stack: Vec::from([(0, TraversalState::Start(start))]),
            _ty: Default::default(),
        }
    }
}
impl<R, Q> NodeCollection<R, Q> for DftStack<R, Q>
    where
        Q: QueryPath,
        R: ResultKind,
{
    fn pop_next(&mut self) -> Option<(usize, TraversalState<R, Q>)> {
        self.stack.pop()
    }
}
impl<R, Q> ExtendStates<R, Q> for DftStack<R, Q>
    where
        Q: QueryPath,
        R: ResultKind,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R, Q>)>,
        T: IntoIterator<Item = (usize, TraversalState<R, Q>), IntoIter=It>
    >(&mut self, iter: T) {
        self.stack.extend(iter.into_iter().rev())
    }
}
impl<R, Q> Default for DftStack<R, Q>
    where
        Q: QueryPath,
        R: ResultKind,
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
            _ty: Default::default(),
        }
    }
}