pub(crate) mod query_range_path;
pub(crate) mod graph_range_path;
pub(crate) mod overlap_primer;
pub(crate) mod prefix_path;
pub(crate) mod traversal;
pub(crate) mod advanceable;
pub(crate) mod reducible;

pub(crate) use query_range_path::*;
pub(crate) use graph_range_path::*;
pub(crate) use overlap_primer::*;
pub(crate) use prefix_path::*;
pub(crate) use traversal::*;
pub(crate) use advanceable::*;
pub(crate) use reducible::*;

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
    fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
        Self::pattern_entry_outer_pos(pattern, self.entry().sub_index)
    }
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
    pub fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self {
        let graph = trav.graph();
        match self {
            StartPath::First { entry, width, .. } => {
                let pattern = graph.expect_pattern_at(entry);
                //println!("first {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                StartPath::Path {
                    entry: parent_entry,
                    path: if entry.sub_index != D::head_index(&pattern) {
                        vec![entry]
                    } else {
                        vec![]
                    },
                    width,
                }
            },
            StartPath::Path { entry, mut path, width } => {
                //println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                let pattern = graph.expect_pattern_at(entry);
                if entry.sub_index != D::head_index(&pattern) || !path.is_empty() {
                    path.push(entry);
                }
                StartPath::Path {
                    entry: parent_entry,
                    path,
                    width,
                }
            },
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

pub trait EntryPos {
    fn get_entry_pos(&self) -> usize;
}
pub trait ExitPos {
    fn get_exit_pos(&self) -> usize;
}
pub trait PatternEntry: EntryPos {
    fn get_entry_pattern(&self) -> &[Child];
    fn get_entry(&self) -> Child {
        self.get_entry_pattern()[self.get_entry_pos()]
    }
}
pub trait PatternExit: ExitPos {
    fn get_exit_pattern(&self) -> &[Child];
    fn get_exit(&self) -> Child {
        self.get_exit_pattern()[self.get_exit_pos()]
    }
}
pub trait GraphEntry: EntryPos {
    fn get_entry_location(&self) -> ChildLocation;
    fn get_entry_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_entry_location())
    }
    fn get_entry<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_entry_location())
    }
}
pub trait GraphExit: ExitPos {
    fn get_exit_location(&self) -> ChildLocation;
    fn get_exit_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_exit_location())
    }
    fn get_exit<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_exit_location())
    }
}
pub trait HasStartPath {
    fn get_start_path(&self) -> &[ChildLocation];
}
pub trait HasEndPath {
    fn get_end_path(&self) -> &[ChildLocation];
}
pub trait PathFinished {
    fn is_finished(&self) -> bool;
    fn set_finished(&mut self);
}
pub trait PatternStart: PatternEntry + HasStartPath {
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.get_start_path().last() {
            trav.graph().expect_child_at(next)
        } else {
            self.get_entry()
        }
    }
}
pub trait PatternEnd: PatternExit + HasEndPath + End {
    fn get_pattern_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(start) = self.get_end_path().last() {
            trav.graph().expect_child_at(start)
        } else {
            self.get_exit()
        }
    }
}
pub trait GraphStart: GraphEntry + HasStartPath {
    fn get_start_location(&self) -> ChildLocation {
        if let Some(start) = self.get_start_path().last() {
            start.clone()
        } else {
            self.get_entry_location()
        }
    }
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_start_location())
    }
}
pub trait GraphEnd: GraphExit + HasEndPath + End {
    fn get_end_location(&self) -> ChildLocation {
        if let Some(end) = self.get_end_path().last() {
            end.clone()
        } else {
            self.get_exit_location()
        }
    }
    fn get_graph_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
}
pub trait EndPathMut {
    fn end_path_mut(&mut self) -> &mut ChildPath;
    fn push_end(&mut self, next: ChildLocation) {
        self.end_path_mut().push(next)
    }
}
pub trait ExitMut: ExitPos {
    fn exit_mut(&mut self) -> &mut usize;
}
pub trait AdvanceableExit: ExitPos + ExitMut + PathFinished {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize>;
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav) {
            *self.exit_mut() = next;
            Ok(())
        } else {
            self.set_finished();
            Err(())
        }
    }
}
impl<P: ExitMut + PatternExit + PathFinished> AdvanceableExit for P {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize> {
        D::pattern_index_next(self.get_exit_pattern(), self.get_exit_pos())
    }
}
pub trait End {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child;
}