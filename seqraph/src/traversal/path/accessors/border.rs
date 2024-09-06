use crate::{
    graph::{
        direction::r#match::MatchDirection,
        kind::GraphKind,
    },
    traversal::{
        path::accessors::role::{
            End,
            Start,
        },
        traversable::Traversable,
    },
};
use std::borrow::Borrow;
use crate::graph::vertex::location::child::ChildLocation;

pub trait RelativeDirection<D: MatchDirection> {
    type Direction: MatchDirection;
}

#[derive(Default)]
pub struct Front;

impl<D: MatchDirection> RelativeDirection<D> for Front {
    type Direction = D;
}

#[derive(Default)]
pub struct Back;

impl<D: MatchDirection> RelativeDirection<D> for Back {
    type Direction = <D as MatchDirection>::Opposite;
}

pub trait PathBorder {
    type BorderDirection<D: MatchDirection>: RelativeDirection<D>;

    //fn pattern_entry_outer_pos<P: IntoPattern>(pattern: P, entry: usize) -> Option<usize> {
    //    <Self::BorderDirection as RelativeDirection<D>>::Direction::pattern_index_next(pattern, entry)
    //}
    //fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
    //    Self::pattern_entry_outer_pos(pattern, <_ as GraphRootChild<R>>::root_child_location(self).sub_index)
    //}
    fn is_at_border<Trav: Traversable>(
        trav: Trav,
        location: ChildLocation,
    ) -> bool {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(location);
        <Self::BorderDirection<<Trav::Kind as GraphKind>::Direction> as RelativeDirection<_>>::Direction::pattern_index_next(
            pattern.borrow(),
            location.sub_index,
        ).is_none()
    }
    //fn is_complete_in_pattern<P: IntoPattern>(&self, pattern: P) -> bool {
    //    self.single_path().is_empty() && self.is_at_pattern_border(pattern)
    //}
}

impl PathBorder for Start {
    type BorderDirection<D: MatchDirection> = Back;
}

impl PathBorder for End {
    type BorderDirection<D: MatchDirection> = Front;
}
//impl<D: MatchDirection> PathBorder<D> for EndPath {
//    type BorderDirection = Front;
//}
