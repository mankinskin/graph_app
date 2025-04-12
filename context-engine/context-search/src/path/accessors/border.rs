use crate::{
    direction::{
        pattern::PatternDirection,
        Direction,
        Left,
        Right,
    },
    graph::vertex::{
        location::child::ChildLocation,
        pattern::Pattern,
    },
    path::accessors::role::{
        End,
        Start,
    },
    traversal::traversable::Traversable,
};

pub trait RelativeDirection: PatternDirection {}

#[derive(Default, Debug, Clone, Copy)]
pub struct Front;

impl Direction for Front {
    type Opposite = Back;
}
impl RelativeDirection for Front {}
impl PatternDirection for Front {
    fn head_index(pattern: &Pattern) -> usize {
        <Right as PatternDirection>::head_index(pattern)
    }
    fn index_next(index: usize) -> Option<usize> {
        <Right as PatternDirection>::index_next(index)
    }
    fn index_prev(index: usize) -> Option<usize> {
        <Right as PatternDirection>::index_prev(index)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Back;

impl RelativeDirection for Back {}

impl Direction for Back {
    type Opposite = Front;
}

impl PatternDirection for Back {
    fn head_index(pattern: &Pattern) -> usize {
        <Left as PatternDirection>::head_index(pattern)
    }
    fn index_next(index: usize) -> Option<usize> {
        <Left as PatternDirection>::index_next(index)
    }
    fn index_prev(index: usize) -> Option<usize> {
        <Left as PatternDirection>::index_prev(index)
    }
}

pub trait PathBorder {
    type BorderDirection: PatternDirection;

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
        <Self::BorderDirection as PatternDirection>::pattern_index_next(pattern, location.sub_index)
            .is_none()
    }
    //fn is_complete_in_pattern<P: IntoPattern>(&self, pattern: P) -> bool {
    //    self.single_path().is_empty() && self.is_at_pattern_border(pattern)
    //}
}

impl PathBorder for Start {
    type BorderDirection = Back;
}

impl PathBorder for End {
    type BorderDirection = Front;
}
//impl<D: > PathBorder<D> for EndPath {
//    type BorderDirection = Front;
//}
