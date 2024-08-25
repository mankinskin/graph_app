use crate::{
    graph::{
        direction::r#match::MatchDirection,
        kind::DirectionOf,
    },
    traversal::traversable::Traversable,
};
use itertools::Itertools;
use std::{
    borrow::Borrow,
    collections::VecDeque,
};
use crate::graph::vertex::{
    child::Child,
    location::{
        child::ChildLocation,
        pattern::PatternLocation,
    },
    pattern::IntoPattern,
    wide::Wide,
};

pub trait BandExpandingPolicy<Trav: Traversable> {
    fn map_band(
        location: PatternLocation,
        pattern: impl IntoPattern,
    ) -> (ChildLocation, Child);
    fn map_batch(
        batch: impl IntoIterator<Item = (ChildLocation, Child)>
    ) -> Vec<(ChildLocation, Child)> {
        batch.into_iter().collect_vec()
    }
}

pub struct PostfixExpandingPolicy<D: MatchDirection> {
    _ty: std::marker::PhantomData<D>,
}

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

impl<Trav: Traversable, D: MatchDirection> BandExpandingPolicy<Trav> for PostfixExpandingPolicy<D> {
    //
    fn map_band(
        location: PatternLocation,
        pattern: impl IntoPattern,
    ) -> (ChildLocation, Child) {
        let last = D::last_index(pattern.borrow());
        (location.to_child_location(last), pattern.borrow()[last])
    }
    fn map_batch(
        batch: impl IntoIterator<Item = (ChildLocation, Child)>
    ) -> Vec<(ChildLocation, Child)> {
        batch
            .into_iter()
            .sorted_by(|a, b| b.1.width().cmp(&a.1.width()))
            .collect_vec()
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

impl<'a, Trav, P> Iterator for BandExpandingIterator<'a, Trav, P>
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
        self.queue
            .pop_front()
            .map(|(location, node)| {
                self.last.0 = Some(location);
                self.last.1 = node;
                (location, node)
            })
            .map(|(location, node)| (location, node))
    }
}
