use crate::{
    direction::r#match::MatchDirection,
    graph::kind::DirectionOf,
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
impl<Trav: Traversable, D: MatchDirection> BandExpandingPolicy<Trav> for PostfixExpandingPolicy<D> {
    //
    fn map_band(
        location: PatternLocation,
        pattern: impl IntoPattern,
    ) -> (ChildLocation, Child) {
        let last = D::last_index(&pattern.borrow());
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