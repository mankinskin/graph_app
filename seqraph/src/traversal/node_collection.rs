use super::*;

pub trait ExtendStates {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter=It>
    >(&mut self, iter: T);
}
pub trait NodeCollection:
    ExtendStates
    //From<StartState>
    + Iterator<Item=(usize, TraversalState)>
    + Default
{
    fn clear(&mut self);
}
#[derive(Clone, Debug)]
pub struct PruningState {
    count: usize,
    prune: bool,
}
pub struct OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeCollection,
{
    collection: O,
    pruning_map: PruningMap,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'a S, Trav)>
}
impl<'a, Trav, S, O> PruneStates for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeCollection,
{
    fn clear(&mut self) {
        self.collection.clear();
    }
    fn pruning_map(&mut self) -> &mut PruningMap {
        &mut self.pruning_map
    }
}
pub type PruningMap = HashMap<DirectedKey, PruningState>;
pub trait PruneStates {
    fn clear(&mut self);
    fn pruning_map(&mut self) -> &mut PruningMap;
    fn prune_not_below(&mut self, root: DirectedKey) {
        self.pruning_map().iter_mut()
            .filter(|(k, _)|
                k.index.width > root.index.width ||
                (k.index.width == root.index.width && k.index != root.index)
            )
            .for_each(|(_, v)| {
                v.prune = true;
            });
    }
    fn prune_smaller(&mut self, root: Child) {
        self.pruning_map().iter_mut()
            .filter(|(k, _)|
                k.index.width < root.width ||
                (k.index.width == root.width && k.index != root)
            )
            .for_each(|(_, v)| {
                v.prune = true;
            });
    }
    fn prune_below(&mut self, root: DirectedKey) {
        self.pruning_map().get_mut(&root).map(|entry| entry.prune = true);
    }
}
impl<'a, Trav, S, O> Unpin for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeCollection,
{
}
impl<'a, Trav, S, O> ExtendStates for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: Traversable,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeCollection,
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
        O: NodeCollection,
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
    O: NodeCollection,
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
pub type Bft<'a, Trav, S> = OrderedTraverser<'a, Trav, S, BftQueue>;
#[allow(unused)]
pub type Dft<'a, Trav, S> = OrderedTraverser<'a, Trav, S, DftStack>;

#[derive(Debug)]
pub struct BftQueue {
    queue: BinaryHeap<QueueEntry>,
}
impl NodeCollection for BftQueue {
    fn clear(&mut self) {
        self.queue.clear()
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
impl NodeCollection for DftStack {
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