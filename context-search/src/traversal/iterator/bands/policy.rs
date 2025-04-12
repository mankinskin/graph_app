use crate::{
    direction::{
        pattern::PatternDirection,
        Direction,
    },
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::PatternLocation,
        },
        pattern::Pattern,
        wide::Wide,
    },
    traversal::traversable::Traversable,
};
use itertools::Itertools;

pub trait BandExpandingPolicy<Trav: Traversable> {
    fn map_band(
        location: PatternLocation,
        pattern: &Pattern,
    ) -> (ChildLocation, Child);
    fn map_batch(
        batch: impl IntoIterator<Item = (ChildLocation, Child)>
    ) -> Vec<(ChildLocation, Child)> {
        batch.into_iter().collect_vec()
    }
}
pub struct PostfixExpandingPolicy<D: PatternDirection> {
    _ty: std::marker::PhantomData<D>,
}
impl<Trav: Traversable, D: PatternDirection> BandExpandingPolicy<Trav> for PostfixExpandingPolicy<D>
where
    <D as Direction>::Opposite: PatternDirection,
{
    //
    fn map_band(
        location: PatternLocation,
        pattern: &Pattern,
    ) -> (ChildLocation, Child) {
        let last = D::last_index(&pattern);
        (location.to_child_location(last), pattern[last])
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

pub struct PrefixExpandingPolicy<D: Direction> {
    _ty: std::marker::PhantomData<D>,
}
impl<Trav: Traversable, D: Direction> BandExpandingPolicy<Trav> for PrefixExpandingPolicy<D> {
    fn map_band(
        location: PatternLocation,
        pattern: &Pattern,
    ) -> (ChildLocation, Child) {
        (location.to_child_location(0), pattern[0])
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
