use crate::*;

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

pub trait PathBorder<D: MatchDirection, R>: GraphRootChild<R> + HasSinglePath {
    type BorderDirection: RelativeDirection<D>;

    fn pattern_entry_outer_pos<P: IntoPattern>(pattern: P, entry: usize) -> Option<usize> {
        <Self::BorderDirection as RelativeDirection<D>>::Direction::pattern_index_next(pattern, entry)
    }
    fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
        Self::pattern_entry_outer_pos(pattern, <_ as GraphRootChild<R>>::root_child_location(self).sub_index)
    }
    fn is_at_pattern_border<P: IntoPattern>(&self, pattern: P) -> bool {
        self.pattern_outer_pos(pattern).is_none()
    }
    fn is_complete_in_pattern<P: IntoPattern>(&self, pattern: P) -> bool {
        self.single_path().is_empty() && self.is_at_pattern_border(pattern)
    }
}

impl<D: MatchDirection> PathBorder<D, Start> for RolePath<Start> {
    type BorderDirection = Back;
}
impl<D: MatchDirection> PathBorder<D, End> for RolePath<End> {
    type BorderDirection = Front;
}
//impl<D: MatchDirection> PathBorder<D> for EndPath {
//    type BorderDirection = Front;
//}