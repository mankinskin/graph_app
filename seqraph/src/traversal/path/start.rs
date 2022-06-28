use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn get_start_path(&self) -> &[ChildLocation] {
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
impl From<IndexingPath> for StartLeaf {
    fn from(p: IndexingPath) -> Self {
        p.start
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StartPath {
    Leaf(StartLeaf),
    Path {
        entry: ChildLocation,
        path: ChildPath,
        width: usize
    },
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
impl From<IndexingPath> for StartPath {
    fn from(p: IndexingPath) -> Self {
        p.into_start_path()
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
        StartPath::Path {
            entry: parent_entry,
            path: if self.entry.sub_index != D::head_index(pattern.borrow()) {
                vec![self.entry]
            } else {
                vec![]
            },
            width: self.width,
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
            StartPath::Path { entry, mut path, width } => {
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
                }
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
    fn get_start_path(&self) -> &[ChildLocation] {
        self.path()
    }
}