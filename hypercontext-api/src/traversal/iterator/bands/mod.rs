use crate::{
    graph::{
        kind::DirectionOf,
        vertex::{
            child::Child,
            location::{
                child::ChildLocation,
                pattern::PatternLocation,
            },
        },
    },
    traversal::traversable::Traversable,
};
use policy::{
    BandExpandingPolicy,
    PostfixExpandingPolicy,
};
use std::{
    borrow::Borrow,
    collections::VecDeque,
};

pub mod policy;

pub trait BandIterator<'a, Trav: Traversable + 'a>:
    Iterator<Item = (ChildLocation, Child)>
{
    type Policy: BandExpandingPolicy<Trav>;
    fn new(
        trav: &'a Trav,
        root: Child,
    ) -> Self;
    fn trav(&self) -> &'a Trav;
    /// get all postfixes of index with their locations
    fn next_children(
        &self,
        index: Child,
    ) -> Vec<(ChildLocation, Child)> {
        Self::Policy::map_batch(self.trav().graph().expect_child_patterns(index).iter().map(
            |(pid, pattern)| {
                Self::Policy::map_band(PatternLocation::new(index, *pid), pattern.borrow())
            },
        ))
    }
}

pub struct BandExpandingIterator<'a, Trav, P>
where
    Trav: Traversable,
    P: BandExpandingPolicy<Trav>,
{
    trav: &'a Trav,
    queue: VecDeque<(ChildLocation, Child)>,
    last: (Option<ChildLocation>, Child),
    _ty: std::marker::PhantomData<&'a P>,
}

pub type PostfixIterator<'a, Trav> =
    BandExpandingIterator<'a, Trav, PostfixExpandingPolicy<DirectionOf<Trav>>>;

impl<'a, Trav, P> BandIterator<'a, Trav> for BandExpandingIterator<'a, Trav, P>
where
    Trav: Traversable,
    P: BandExpandingPolicy<Trav>,
{
    type Policy = P;
    fn new(
        trav: &'a Trav,
        root: Child,
    ) -> Self {
        Self {
            trav,
            queue: VecDeque::new(),
            last: (None, root),
            _ty: Default::default(),
        }
    }
    fn trav(&self) -> &'a Trav {
        self.trav
    }
}

impl<Trav, P> Iterator for BandExpandingIterator<'_, Trav, P>
where
    Trav: Traversable,
    P: BandExpandingPolicy<Trav>,
{
    type Item = (ChildLocation, Child);

    fn next(&mut self) -> Option<Self::Item> {
        //let mut segment = None;
        let next = self.next_children(self.last.1);
        if self.queue.is_empty() {
            //segment = last_location.take();
            self.queue.extend(next)
        }
        self.queue.pop_front().map(|(location, node)| {
            self.last.0 = Some(location);
            self.last.1 = node;
            (location, node)
        })
    }
}
