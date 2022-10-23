use std::collections::VecDeque;

use itertools::Itertools;

use crate::{PatternLocation, ChildLocation, Child, Wide};

use super::*;

pub(crate) trait BandExpandingPolicy<
    'a: 'g,
    'g,
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
> {
    fn map_band(location: PatternLocation, pattern: impl IntoPattern) -> (ChildLocation, Child);
    fn map_batch(batch: impl IntoIterator<Item=(ChildLocation, Child)>) -> Vec<(ChildLocation, Child)> {
        batch.into_iter().collect_vec()
    }
}
pub(crate) struct PostfixExpandingPolicy<D: MatchDirection> {
    _ty: std::marker::PhantomData<D>,
}

pub(crate) trait BandIterator<
    'a: 'g,
    'g,
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
>: Iterator<Item = (ChildLocation, Child)>
{
    fn new(trav: &'a Trav, root: Child) -> Self;
    /// get all postfixes of index with their locations
    fn next_children(trav: &'a Trav, index: Child) -> Vec<(ChildLocation, Child)> {
        P::map_batch(
            trav.graph()
                .expect_child_patterns_of(index)
                .iter()
                .map(|(pid, pattern)|
                    P::map_band(PatternLocation::new(index, *pid), pattern.borrow())
                )
        )
    }
}
impl <
    'a: 'g,
    'g,
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
> BandExpandingPolicy<'a, 'g, T, Trav> for PostfixExpandingPolicy<D> {
    // 
    fn map_band(location: PatternLocation, pattern: impl IntoPattern) -> (ChildLocation, Child) {
        let last = D::last_index(pattern.borrow());
        (location.to_child_location(last), pattern.borrow()[last])
    }
    fn map_batch(batch: impl IntoIterator<Item=(ChildLocation, Child)>) -> Vec<(ChildLocation, Child)> {
        batch.into_iter()
            .sorted_by(|a, b|
                b.1.width().cmp(&a.1.width())
            )
            .collect_vec()
    }
}
pub(crate) struct BandExpandingIterator<'a: 'g, 'g, T, Trav, P>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
{
    trav: &'a Trav,
    queue: VecDeque<(ChildLocation, Child)>,
    last: (Option<ChildLocation>, Child),
    _ty: std::marker::PhantomData<(&'g T, P)>
}
pub(crate) type PostfixIterator<'a, 'g, T, D, Trav>
    = BandExpandingIterator<'a, 'g, T, Trav, PostfixExpandingPolicy<D>>;

impl<'a: 'g, 'g, T, Trav, P> BandIterator<'a, 'g, T, Trav, P> for BandExpandingIterator<'a, 'g, T, Trav, P>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
{
    fn new(trav: &'a Trav, root: Child) -> Self {
        Self {
            trav,
            queue: VecDeque::new(),
            last: (None, root),
            _ty: Default::default(),
        }
    }
}
impl<'a: 'g, 'g, T, Trav, P> Iterator for BandExpandingIterator<'a, 'g, T, Trav, P>
where
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
{
    type Item = (ChildLocation, Child);

    fn next(&mut self) -> Option<Self::Item> {
        let (last_location, last_node) = &mut self.last;
        let mut segment = None;
        if self.queue.is_empty() {
            segment = last_location.take();
            self.queue.extend(
                <Self as BandIterator<T, Trav, P>>::next_children(self.trav, *last_node)
            )
        }
        self.queue.pop_front()
            .map(|(location, node)| { 
                *last_location = Some(location);
                *last_node = node;
                (location, node)
            })
    }
}