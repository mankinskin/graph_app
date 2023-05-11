use std::collections::VecDeque;

use itertools::Itertools;

use crate::{PatternLocation, ChildLocation, Child, Wide};

use super::*;

pub trait BandExpandingPolicy<
    Trav: Traversable,
> {
    fn map_band(location: PatternLocation, pattern: impl IntoPattern) -> (ChildLocation, Child);
    fn map_batch(batch: impl IntoIterator<Item=(ChildLocation, Child)>) -> Vec<(ChildLocation, Child)> {
        batch.into_iter().collect_vec()
    }
}
pub struct PostfixExpandingPolicy<D: MatchDirection> {
    _ty: std::marker::PhantomData<D>,
}


pub trait BandIterator<
    'a,
    Trav: Traversable + 'a,
    P: BandExpandingPolicy<Trav>,
>: Iterator<Item = (ChildLocation, Child)>
{
    fn new(trav: &'a Trav, root: Child) -> Self;
    fn trav(&self) -> &'a Trav;
    /// get all postfixes of index with their locations
    fn next_children(&self, index: Child) -> Vec<(ChildLocation, Child)> {
        P::map_batch(
            self.trav().graph()
                .expect_child_patterns(index)
                .iter()
                .map(|(pid, pattern)|
                    P::map_band(
                        PatternLocation::new(index, *pid),
                        pattern.borrow() as &[Child]
                    )
                )
        )
    }
}
impl <
    Trav: Traversable,
    D: MatchDirection,
> BandExpandingPolicy<Trav> for PostfixExpandingPolicy<D> {
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
pub struct BandExpandingIterator<'a, Trav, P>
where
    Trav: Traversable,
    P: BandExpandingPolicy<Trav>,
{
    trav: &'a Trav,
    queue: VecDeque<(ChildLocation, Child)>,
    last: (Option<ChildLocation>, Child),
    _ty: std::marker::PhantomData<&'a P>
}
pub type PostfixIterator<'a, Trav>
    = BandExpandingIterator<'a, Trav, PostfixExpandingPolicy<<Trav as GraphKind>::Direction>>;


impl<'a, Trav, P> BandIterator<'a, Trav, P> for BandExpandingIterator<'a, Trav, P>
where
    Trav: Traversable,
    P: BandExpandingPolicy<Trav>,
{
    fn new(trav: &'a Trav, root: Child) -> Self {
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
            self.queue.extend(
                next
            )
        }
        self.queue.pop_front()
            .map(|(location, node)| { 
                self.last.0 = Some(location);
                self.last.1 = node;
                (location, node)
            })
            .map(|(location, node)|
                (location, node)
            )
        
    }
}