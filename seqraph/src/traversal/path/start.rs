use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct StartLeaf {
    pub(crate) entry: ChildLocation,
    pub(crate) child: Child,
    pub(crate) width: usize,
}
impl WideMut for StartLeaf {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}
impl Wide for StartLeaf {
    fn width(&self) -> usize {
        self.width
    }
}
impl GraphEntry for StartLeaf {
    fn get_entry_location(&self) -> ChildLocation {
        self.entry
    }
}
impl HasStartPath for StartLeaf {
    fn start_path(&self) -> &[ChildLocation] {
        self.path()
    }
}
impl BorderPath for StartLeaf {
    fn path(&self) -> &[ChildLocation] {
        &[]
    }
    fn entry(&self) -> ChildLocation {
        self.get_entry_location()
    }
}
impl<D: MatchDirection> DirectedBorderPath<D> for StartLeaf {
    type BorderDirection = Back;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum StartPath {
    Leaf(StartLeaf),
    Path {
        entry: ChildLocation,
        path: ChildPath,
        width: usize,
        child: Child,
    },
}
impl StartPath {
    #[allow(unused)]
    pub fn get_leaf(&self) -> Option<&StartLeaf> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }
    /// returns child if reduced to single child
    pub fn reduce<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        match self {
            Self::Leaf(leaf) => {
                let graph = trav.graph();
                let pattern = graph.expect_pattern_at(leaf.entry);
                (leaf.entry.sub_index == D::head_index(pattern.borrow()))
                    .then(|| leaf.entry.parent)
            },
            // TODO: maybe skip path segments starting at pattern head
            Self::Path { .. } => None,
        }
    }
}
impl From<SearchPath> for StartPath {
    fn from(p: SearchPath) -> Self {
        p.start
    }
}
impl From<StartLeaf> for StartPath {
    fn from(leaf: StartLeaf) -> Self {
        StartPath::Leaf(leaf)
    }
}
impl WideMut for StartPath {
    fn width_mut(&mut self) -> &mut usize {
        match self {
            Self::Path { width, .. } => width,
            Self::Leaf(leaf) => leaf.width_mut(),
        }
    }
}
pub(crate) trait PathAppend {
    type Result: PathAppend;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result;
}
impl PathAppend for StartLeaf {
    type Result = StartPath;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(self.entry);
        if self.entry.sub_index == D::head_index(pattern.borrow()) {
            StartPath::Leaf(StartLeaf {
                entry: parent_entry,
                child: self.entry.parent,
                width: self.width,
            })
        } else {
            StartPath::Path {
                entry: parent_entry,
                path: vec![self.entry],
                width: self.width,
                child: self.child,
            }
        }
    }
}
impl PathAppend for StartPath {
    type Result = Self;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        match self {
            StartPath::Leaf(leaf) => leaf.append::<_, D, _>(trav, parent_entry),
            StartPath::Path { entry, mut path, width , child} => {
                let graph = trav.graph();
                //println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                let pattern = graph.expect_pattern_at(entry);
                if entry.sub_index != D::head_index(pattern.borrow()) || !path.is_empty() {
                    path.push(entry);
                }
                StartPath::Path {
                    entry: parent_entry,
                    path,
                    width,
                    child,
                }
            },
        }
    }
}
pub(crate) trait PathPop {
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav) -> MatchEnd;
}
impl PathPop for StartPath {
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav) -> MatchEnd {
        match self {
            StartPath::Leaf(leaf) => MatchEnd::Complete(leaf.child),
            StartPath::Path { entry, mut path, width, child } => {
                MatchEnd::Path(if let Some(seg) = path.pop() {
                    if path.is_empty() {
                        StartPath::Leaf(StartLeaf {
                            entry: seg,
                            child,
                            width,
                        })
                    } else {
                        StartPath::Path {
                            entry: seg,
                            path,
                            width,
                            child,
                        }
                    }
                } else {
                    let graph = trav.graph();
                    StartPath::Leaf(StartLeaf {
                        child: graph.expect_child_at(&entry),
                        entry,
                        width,
                    })
                })
            },
        }
    }
}
impl BorderPath for StartPath {
    fn path(&self) -> &[ChildLocation] {
        match self {
            StartPath::Leaf(leaf) => leaf.path(),
            StartPath::Path{ path, ..} => path.borrow(),
        }
    }
    fn entry(&self) -> ChildLocation {
        self.get_entry_location()
    }
}
impl<D: MatchDirection> DirectedBorderPath<D> for StartPath {
    type BorderDirection = Back;
}
impl Wide for StartPath {
    fn width(&self) -> usize {
        match self {
            Self::Path { width, .. } |
            Self::Leaf(StartLeaf { width, .. }) => *width,
        }
    }
}
impl GraphEntry for StartPath {
    fn get_entry_location(&self) -> ChildLocation {
        match self {
            Self::Path { entry, .. } |
            Self::Leaf(StartLeaf { entry, .. })
                => *entry,
        }
    }
}
impl HasStartPath for StartPath {
    fn start_path(&self) -> &[ChildLocation] {
        self.path()
    }
}