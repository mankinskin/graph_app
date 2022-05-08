pub(crate) mod range_path;
pub(crate) mod query_range_path;
pub(crate) mod graph_range_path;
pub(crate) mod overlap_primer;
pub(crate) mod prefix_path;
pub(crate) mod traversal;

pub(crate) use range_path::*;
pub(crate) use query_range_path::*;
pub(crate) use graph_range_path::*;
pub(crate) use overlap_primer::*;
pub(crate) use prefix_path::*;
pub(crate) use traversal::*;

use crate::{
    vertex::*,
    *,
};
pub trait RelativeDirection {
    type Direction: MatchDirection;
}
#[derive(Default)]
pub(crate) struct Front<D: MatchDirection>(std::marker::PhantomData<D>);
impl<D: MatchDirection> RelativeDirection for Front<D> {
    type Direction = D;
}
#[derive(Default)]
pub(crate) struct Back<D: MatchDirection>(std::marker::PhantomData<D>);
impl<D: MatchDirection> RelativeDirection for Back<D> {
    type Direction = <D as MatchDirection>::Opposite;
}

pub(crate) trait BorderPath: Wide {
    fn entry(&self) -> ChildLocation;
    fn path(&self) -> &[ChildLocation];
    /// true if path points to direct border in entry (path is empty)
    fn is_perfect(&self) -> bool {
        self.path().is_empty()
    }
    fn pattern<'a: 'g, 'g, 'x, T: Tokenize + 'g, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> Pattern {
        let graph = trav.graph();
        graph.expect_pattern_at(&self.entry())
    }
}
pub(crate) trait DirectedBorderPath<D: MatchDirection>: BorderPath {
    type BorderDirection: RelativeDirection;
    fn pattern_entry_outer_pos<P: IntoPattern>(pattern: P, entry: usize) -> Option<usize> {
        <Self::BorderDirection as RelativeDirection>::Direction::pattern_index_next(pattern, entry)
    }
    //fn pattern_entry_outer_context<P: IntoPattern>(pattern: P, entry: usize) -> ContextHalf {
    //    ContextHalf::try_new(<Self::BorderDirection as RelativeDirection>::Direction::front_context(pattern.borrow(), entry))
    //        .expect("GraphPath references border of index!")
    //}
    //fn pattern_outer_context<P: IntoPattern>(&self, pattern: P) -> ContextHalf {
    //    Self::pattern_entry_outer_context(pattern, self.entry().sub_index)
    //}
    fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
        Self::pattern_entry_outer_pos(pattern, self.entry().sub_index)
    }
    //fn outer_context<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> ContextHalf {
    //    self.pattern_outer_context(self.pattern(trav))
    //}
    fn outer_pos<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> Option<usize> {
        self.pattern_outer_pos(self.pattern(trav))
    }
    fn is_at_pattern_border<P: IntoPattern>(&self, pattern: P) -> bool {
        self.pattern_outer_pos(pattern).is_none()
    }
    fn pattern_is_complete<P: IntoPattern>(&self, pattern: P) -> bool {
        self.is_perfect() && self.is_at_pattern_border(pattern)
    }
    fn is_at_border<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> bool {
        self.outer_pos(trav).is_none()
    }
    fn is_complete<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> bool {
        self.is_perfect() && self.is_at_border(trav)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EndPath {
    pub(crate) entry: ChildLocation,
    pub(crate) path: ChildPath,
    pub(crate) width: usize,
}
//impl EndPath {
//    pub fn path_mut(&mut self) -> &mut ChildPath {
//        &mut self.path
//    }
//    pub fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}
impl BorderPath for EndPath {
    fn entry(&self) -> ChildLocation {
        self.entry
    }
    fn path(&self) -> &[ChildLocation] {
        self.path.as_slice()
    }
}
impl<D: MatchDirection> DirectedBorderPath<D> for EndPath {
    type BorderDirection = Front<D>;
}
impl Wide for EndPath {
    fn width(&self) -> usize {
        self.width
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StartPath {
    First {
        entry: ChildLocation,
        child: Child,
        width: usize,
    },
    Path {
        entry: ChildLocation,
        path: ChildPath,
        width: usize
    },
}
impl StartPath {
    pub fn width_mut(&mut self) -> &mut usize {
        match self {
            Self::Path { width, .. } |
            Self::First { width , ..} => width,
        }
    }
}
impl BorderPath for StartPath {
    fn entry(&self) -> ChildLocation {
        match self {
            Self::Path { entry, .. } |
            Self::First { entry, .. }
                => *entry,
        }
    }
    fn path(&self) -> &[ChildLocation] {
        match self {
            Self::Path { path, .. } => path.as_slice(),
            _ => &[],
        }
    }
}
impl<D: MatchDirection> DirectedBorderPath<D> for StartPath {
    type BorderDirection = Back<D>;
}
impl Wide for StartPath {
    fn width(&self) -> usize {
        match self {
            Self::Path { width, .. } |
            Self::First { width, .. } => *width,
        }
    }
}