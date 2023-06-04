use super::*;
pub mod pruning;
pub use pruning::*;

pub trait ExtendStates {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter=It>
    >(&mut self, iter: T);
}
pub trait NodeVisitor:
    ExtendStates
    + Iterator<Item=(usize, TraversalState)>
    + Default
{
    fn clear(&mut self);
}
pub struct OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeVisitor,
{
    collection: O,
    pruning_map: PruningMap,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'a S, Trav)>
}
impl<'a, Trav, S, O> Unpin for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeVisitor,
{
}
impl<'a, Trav, S, O> ExtendStates for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeVisitor,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        In: IntoIterator<Item = (usize, TraversalState), IntoIter=It>
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
impl<'a, Trav, S, O> TraversalIterator<'a, Trav, S> for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable + 'a + TraversalFolder<S>,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeVisitor,
{
    fn new(trav: &'a Trav) -> Self {
        Self {
            //pruning_map: HashMap::from([
            //    (DirectedKey::new(start.index, 0), PruningState {
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
impl<'a, Trav, S, O> Iterator for OrderedTraverser<'a, Trav, S, O>
where
    Trav: Traversable + TraversalFolder<S>,
    S: DirectedTraversalPolicy<Trav=Trav>,
    O: NodeVisitor,
{
    type Item = (usize, TraversalState);

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

pub trait TraversalOrder: Wide {
    fn sub_index(&self) -> usize;
    fn cmp(&self, other: impl TraversalOrder) -> Ordering {
        match other.width().cmp(&self.width()) {
            Ordering::Equal => self.sub_index().cmp(&other.sub_index()),
            r => r,
        }
    }
}
impl<T: TraversalOrder> TraversalOrder for &T {
    fn sub_index(&self) -> usize {
        TraversalOrder::sub_index(*self)
    }
}
impl TraversalOrder for ChildLocation {
    fn sub_index(&self) -> usize {
        self.sub_index
    }
}

pub type Bft<'a, Trav, S> = OrderedTraverser<'a, Trav, S, BftQueue>;
#[derive(Debug)]
pub struct BftQueue {
    queue: BinaryHeap<QueueEntry>,
}
impl NodeVisitor for BftQueue {
    fn clear(&mut self) {
        self.queue.clear()
    }
}
impl Iterator for BftQueue {
    type Item = (usize, TraversalState);
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|QueueEntry(d, s)| (d, s))
    }
}
impl ExtendStates for BftQueue
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter=It>
    >(&mut self, iter: T) {
        self.queue.extend(iter.into_iter().map(|(d, s)| QueueEntry(d, s)))
    }
}
impl Default for BftQueue
{
    fn default() -> Self {
        Self {
            queue: Default::default(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct QueueEntry(usize, TraversalState);

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0).then_with(||
            self.1.cmp(&other.1)
        )
    }
}
impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(unused)]
pub type Dft<'a, Trav, S> = OrderedTraverser<'a, Trav, S, DftStack>;

#[derive(Debug)]
pub struct DftStack
{
    stack: Vec<(usize, TraversalState)>,
}
//impl From<StartState> for DftStack {
//    fn from(start: StartState) -> Self {
//        Self {
//            stack: Vec::from([(0, TraversalState::Start(start))]),
//            _ty: Default::default(),
//        }
//    }
//}
impl NodeVisitor for DftStack {
    fn clear(&mut self) {
        self.stack.clear()
    }
}
impl Iterator for DftStack
{
    type Item = (usize, TraversalState);
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}
impl ExtendStates for DftStack
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter=It>
    >(&mut self, iter: T) {
        self.stack.extend(iter.into_iter().rev())
    }
}
impl Default for DftStack
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}