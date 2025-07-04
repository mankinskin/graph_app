use crate::{
    direction::{
        Direction,
        pattern::PatternDirection,
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
    trace::has_graph::HasGraph,
};
use itertools::Itertools;

pub trait BandExpandingPolicy<G: HasGraph> {
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
#[derive(Debug)]
pub struct PostfixExpandingPolicy<D: PatternDirection> {
    _ty: std::marker::PhantomData<D>,
}
impl<G: HasGraph, D: PatternDirection> BandExpandingPolicy<G>
    for PostfixExpandingPolicy<D>
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

#[derive(Debug)]
pub struct PrefixExpandingPolicy<D: Direction> {
    _ty: std::marker::PhantomData<D>,
}
impl<G: HasGraph, D: Direction> BandExpandingPolicy<G>
    for PrefixExpandingPolicy<D>
{
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
