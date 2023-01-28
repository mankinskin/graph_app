use std::collections::VecDeque;
use crate::{
    Tokenize,
    MatchDirection,
};
use std::iter::Extend;

use super::*;

pub trait ExtendStates<
    R: ResultKind,
>
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R>)>,
        T: IntoIterator<Item = (usize, TraversalState<R>), IntoIter=It>
    >(&mut self, iter: T);
}
pub trait NodeCollection<
    R: ResultKind,
>:
    ExtendStates<R>
    //From<StartState<R>>
    + Iterator<Item=(usize, TraversalState<R>)>
    + Default
    where
        R: ResultKind,
{
}
impl<
    R: ResultKind,
    T:
    ExtendStates<R>
    //From<StartState<R>>
    + Iterator<Item=(usize, TraversalState<R>)>
    + Default
> NodeCollection<R> for T
{
}
#[derive(Clone, Debug)]
pub struct PruningState {
    count: usize,
    prune: bool,
}
pub struct OrderedTraverser<'a, T, D, Trav, R, S, O>
    where
        T: Tokenize,
        Trav: Traversable<T>,
        D: MatchDirection,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
        O: NodeCollection<R>,
{
    collection: O,
    pruning_map: HashMap<CacheKey, PruningState>,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'a T, D, S, R, Trav)>
}
impl<'a, T, D, Trav, R, S, O> OrderedTraverser<'a, T, D, Trav, R, S, O>
    where
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
        O: NodeCollection<R>,
{
    pub fn prune_not_below(&mut self, root: CacheKey) {
        self.pruning_map.iter_mut()
            .filter(|(k, _)| k.index.width >= root.index.width)
            .for_each(|(_, v)| {
                v.prune = true;
            });
    }
    pub fn prune_below(&mut self, root: CacheKey) {
        self.pruning_map.get_mut(&root).unwrap().prune = true;
    }
}
impl<'a, T, D, Trav, R, S, O> Unpin for OrderedTraverser<'a, T, D, Trav, R, S, O>
    where
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
        O: NodeCollection<R>,
{
}
impl<'a, T, D, Trav, R, S, O> ExtendStates<R> for OrderedTraverser<'a, T, D, Trav, R, S, O>
    where
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
        O: NodeCollection<R>,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R>)>,
        In: IntoIterator<Item = (usize, TraversalState<R>), IntoIter=It>
    >(&mut self, iter: In) {
        let states = iter.into_iter().map(|(d, s)| {
                // count states per root
                self.pruning_map.entry(s.root_key())
                    .and_modify(|ps| ps.count = ps.count + 1)
                    .or_insert(PruningState {
                        count: 1,
                        prune: false,
                    });
                (d, s)
            })
            .collect_vec();
        self.collection.extend(
            states
        )
    }
}
impl<'a, T, Trav, D, S, R, O> TraversalIterator<'a, T, D, Trav, S, R> for OrderedTraverser<'a, T, D, Trav, R, S, O>
    where
        T: Tokenize,
        Trav: Traversable<T> + 'a + TraversalFolder<T, D, S, R>,
        D: MatchDirection,
        R: ResultKind,
        S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
        O: NodeCollection<R>,
{
    fn new(trav: &'a Trav) -> Self {
        Self {
            //pruning_map: HashMap::from([
            //    (CacheKey::new(start.index, 0), PruningState {
            //        count: 1,
            //        prune: false,
            //    })
            //]),
            //collection: O::from(start),
            pruning_map: Default::default(),
            collection: Default::default(),
            trav,
            _ty: Default::default(),
        }
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
}
impl<'a, T, D, Trav, R, S, O> Iterator for OrderedTraverser<'a, T, D, Trav, R, S, O>
where
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T> + TraversalFolder<T, D, S, R>,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, R, Trav=Trav>,
    O: NodeCollection<R>,
{
    type Item = (usize, TraversalState<R>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((d, s)) = self.collection.next() {
            let mut e = self.pruning_map.get_mut(&s.root_key()).unwrap();
            e.count = e.count - 1;
            let pass = !e.prune;
            if e.count == 0 {
                self.pruning_map.remove(&s.root_key());
            }
            if pass {
                return Some((d, s))
            }
        }
        None
    }
}
pub type Bft<'a, T, D, Trav, R, S> = OrderedTraverser<'a, T, D, Trav, R, S, BftQueue<R>>;
#[allow(unused)]
pub type Dft<'a, T, D, Trav, R, S> = OrderedTraverser<'a, T, D, Trav, R, S, DftStack<R>>;

#[derive(Debug)]
pub struct BftQueue<R>
    where
        R: ResultKind,
{
    queue: VecDeque<(usize, TraversalState<R>)>,
    _ty: std::marker::PhantomData<R>
}
//impl<R: ResultKind> From<StartState<R>> for BftQueue<R> {
//    fn from(start: StartState<R>) -> Self {
//        Self {
//            queue: VecDeque::from([(0, TraversalState::Start(start))]),
//            _ty: Default::default(),
//        }
//    }
//}
impl<R> Iterator for BftQueue<R>
    where
        R: ResultKind,
{
    type Item = (usize, TraversalState<R>);
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}
impl<R> ExtendStates<R> for BftQueue<R>
    where
        R: ResultKind,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R>)>,
        T: IntoIterator<Item = (usize, TraversalState<R>), IntoIter=It>
    >(&mut self, iter: T) {
        self.queue.extend(iter)
    }
}
impl<R> Default for BftQueue<R>
    where
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
pub struct DftStack<R>
where
    R: ResultKind,
{
    stack: Vec<(usize, TraversalState<R>)>,
    _ty: std::marker::PhantomData<R>
}
//impl<R: ResultKind> From<StartState<R>> for DftStack<R> {
//    fn from(start: StartState<R>) -> Self {
//        Self {
//            stack: Vec::from([(0, TraversalState::Start(start))]),
//            _ty: Default::default(),
//        }
//    }
//}
impl<R> Iterator for DftStack<R>
    where
        R: ResultKind,
{
    type Item = (usize, TraversalState<R>);
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}
impl<R> ExtendStates<R> for DftStack<R>
    where
        R: ResultKind,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState<R>)>,
        T: IntoIterator<Item = (usize, TraversalState<R>), IntoIter=It>
    >(&mut self, iter: T) {
        self.stack.extend(iter.into_iter().rev())
    }
}
impl<R> Default for DftStack<R>
    where
        R: ResultKind,
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
            _ty: Default::default(),
        }
    }
}