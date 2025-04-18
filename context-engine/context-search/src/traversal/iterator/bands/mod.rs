use crate::traversal::Traversable;
use context_trace::graph::{
    kind::DirectionOf,
    vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::PatternLocation,
        },
    },
};
use policy::{
    BandExpandingPolicy,
    PostfixExpandingPolicy,
    PrefixExpandingPolicy,
};
use std::collections::VecDeque;

pub mod policy;

pub trait BandIterator<'a, Trav: Traversable + 'a>:
    Iterator<Item = (ChildLocation, Child)>
{
    type Policy: BandExpandingPolicy<Trav>;
    fn band_iter(
        trav: Trav,
        root: Child,
    ) -> Self;
    fn trav(&self) -> &Trav;
    fn trav_mut(&mut self) -> &mut Trav;
    /// get all postfixes of index with their locations
    fn next_children(
        &self,
        index: Child,
    ) -> Vec<(ChildLocation, Child)> {
        Self::Policy::map_batch(
            self.trav().graph().expect_child_patterns(index).iter().map(
                |(pid, pattern)| {
                    Self::Policy::map_band(
                        PatternLocation::new(index, *pid),
                        &pattern,
                    )
                },
            ),
        )
    }
}

pub struct BandExpandingIterator<'a, Trav, P>
where
    Trav: Traversable + 'a,
    P: BandExpandingPolicy<Trav>,
{
    trav: Trav,
    queue: VecDeque<(ChildLocation, Child)>,
    last: (Option<ChildLocation>, Child),
    _ty: std::marker::PhantomData<&'a P>,
}

pub type PostfixIterator<'a, Trav> = BandExpandingIterator<
    'a,
    Trav,
    PostfixExpandingPolicy<DirectionOf<<Trav as Traversable>::Kind>>,
>;

pub type PrefixIterator<'a, Trav> = BandExpandingIterator<
    'a,
    Trav,
    PrefixExpandingPolicy<DirectionOf<<Trav as Traversable>::Kind>>,
>;

impl<'a, Trav, P> BandIterator<'a, Trav> for BandExpandingIterator<'a, Trav, P>
where
    Trav: Traversable + 'a,
    P: BandExpandingPolicy<Trav>,
{
    type Policy = P;
    fn band_iter(
        trav: Trav,
        root: Child,
    ) -> Self {
        Self {
            trav,
            queue: VecDeque::new(),
            last: (None, root),
            _ty: Default::default(),
        }
    }
    fn trav(&self) -> &Trav {
        &self.trav
    }
    fn trav_mut(&mut self) -> &mut Trav {
        &mut self.trav
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
        self.queue.pop_front().map(|(location, node)| {
            self.last.0 = Some(location);
            self.last.1 = node;
            (location, node)
        })
    }
}
