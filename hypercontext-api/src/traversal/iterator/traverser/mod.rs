
use extend::ExtendStates;
use pruning::PruningMap;
use std::{
    cmp::Ordering,
    fmt::Debug,
};

pub mod dft;
pub mod bft;
pub mod pruning;
pub mod extend;

use crate::{graph::vertex::wide::Wide, traversal::{
    cache::{
        key::root::RootKey, state::traversal::TraversalState,
    },
    context::TraversalStateContext,
    folder::TraversalFolder,
    iterator::TraversalIterator,
    policy::DirectedTraversalPolicy,
    traversable::Traversable,
}};
use crate::graph::vertex::location::child::ChildLocation;

pub trait TraversalOrder: Wide {
    fn sub_index(&self) -> usize;
    fn cmp(
        &self,
        other: impl TraversalOrder,
    ) -> Ordering {
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

pub trait NodeVisitor:
    ExtendStates + Iterator<Item = (usize, TraversalState)> + Default + Debug
{
    fn clear(&mut self);
}

#[derive(Debug)]
pub struct OrderedTraverser<'a, Trav, S, O>
where
    Trav: Traversable,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
    pub collection: O,
    pub pruning_map: PruningMap,
    pub trav: &'a Trav,
    pub _ty: std::marker::PhantomData<(&'a S, Trav)>,
}

impl<'a, Trav, S, O> From<&'a Trav> for OrderedTraverser<'a, Trav, S, O>
where
    Trav: Traversable,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
    fn from(trav: &'a Trav) -> Self {
        Self {
            pruning_map: Default::default(),
            collection: Default::default(),
            trav,
            _ty: Default::default(),
        }
    }
}

impl<Trav, S, O> Unpin for OrderedTraverser<'_, Trav, S, O>
where
    Trav: Traversable,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
}

impl<Trav, S, O> Iterator for OrderedTraverser<'_, Trav, S, O>
where
    Trav: Traversable + TraversalFolder,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
    type Item = (usize, TraversalState);

    fn next(&mut self) -> Option<Self::Item> {
        for (d, s) in self.collection.by_ref() {
            let e = self.pruning_map.get_mut(&s.root_key()).unwrap();
            e.count -= 1;
            let pass = !e.prune;
            if e.count == 0 {
                self.pruning_map.remove(&s.root_key());
            }
            if pass {
                return Some((d, s));
            }
        }
        None
    }
}

impl<'a, 'b: 'a, I: TraversalIterator<'b>> Iterator for TraversalStateContext<'a, 'b, I> {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
