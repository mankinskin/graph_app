use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct StartLeaf {
    pub(crate) entry: ChildLocation,
    pub(crate) child: Child,
    pub(crate) width: usize,
}
impl StartLeaf {
    pub fn new(child: Child, entry: ChildLocation) -> Self {
        Self {
            entry,
            child,
            width: child.width(),
        }
    }
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
    fn entry(&self) -> ChildLocation {
        self.entry
    }
}
impl HasStartPath for StartLeaf {
    fn start_path(&self) -> &[ChildLocation] {
        &[]
    }
}
impl PathRoot for StartLeaf {
    fn root(&self) -> ChildLocation {
        self.entry()
    }
}
impl From<StartPath> for StartLeaf {
    fn from(path: StartPath) -> Self {
        match path {
            StartPath::Leaf(leaf) => leaf,
            StartPath::Path {
                entry,
                child,
                width,
                ..
            } => StartLeaf {
                entry,
                child,
                width,
            }
        }
    }
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
    #[allow(unused)]
    pub fn into_path(self) -> ChildPath {
        match self {
            Self::Leaf(_leaf) => Vec::new(),
            Self::Path { path, .. } => path,
        }
    }
    pub fn into_context_path(self) -> ChildPath {
        match self {
            Self::Leaf(leaf) => vec![leaf.entry],
            Self::Path {
                entry,
                path,
                ..
            } => path.tap_mut(|p|
                p.push(entry)
            ),
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
    type Result;
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
    type Result;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav) -> Self::Result;
}
impl PathPop for OriginPath<SearchPath> {
    type Result = OriginPath<<SearchPath as PathPop>::Result>;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav) -> Self::Result {
        OriginPath {
            postfix: self.postfix.pop_path::<_, D, _>(trav),
            origin: self.origin.pop_path::<_, D, _>(trav)
                .unwrap_or_else(|err| MatchEnd::Complete(err))
        }
    }
}
impl PathPop for SearchPath {
    type Result = <StartPath as PathPop>::Result;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav) -> Self::Result {
        self.start.pop_path::<_, D, _>(trav)
    }
}
impl PathPop for StartPath {
    type Result = MatchEnd<StartPath>;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav) -> Self::Result {
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
impl<D: MatchDirection> PathBorder<D> for StartPath {
    type BorderDirection = Back;
}
impl HasSinglePath for StartPath {
    fn single_path(&self) -> &[ChildLocation] {
        self.start_path()
    }
}
impl Wide for StartPath {
    fn width(&self) -> usize {
        match self {
            Self::Path { width, .. } |
            Self::Leaf(StartLeaf { width, .. }) => *width,
        }
    }
}
impl PathRoot for StartPath {
    fn root(&self) -> ChildLocation {
        self.entry()
    }
}
impl GraphEntry for StartPath {
    fn entry(&self) -> ChildLocation {
        match self {
            Self::Path { entry, .. } |
            Self::Leaf(StartLeaf { entry, .. })
                => *entry,
        }
    }
}
impl HasStartPath for StartPath {
    fn start_path(&self) -> &[ChildLocation] {
        match self {
            StartPath::Leaf(leaf) => leaf.start_path(),
            StartPath::Path{ path, ..} => path.borrow(),
        }
    }
}