use crate::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PathLeaf {
    pub entry: ChildLocation,
    pub child: Child,
    pub width: usize,
    pub token_pos: usize,
}
impl PathLeaf {
    pub fn new(child: Child, entry: ChildLocation) -> Self {
        Self {
            entry,
            child,
            width: child.width(),
            token_pos: child.width(),
        }
    }
}
impl WideMut for PathLeaf {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}
impl Wide for PathLeaf {
    fn width(&self) -> usize {
        self.width
    }
}
impl PathLeaf {
    pub fn path(&self) -> &Vec<ChildLocation> {
        &Vec::new()
    }
    pub fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut Vec::new()
    }
    pub fn child_location(&self) -> ChildLocation {
        self.entry
    }
    pub fn child_location_mut(&mut self) -> &mut ChildLocation {
        &mut self.entry
    }
}
impl From<ChildPath> for PathLeaf {
    fn from(path: ChildPath) -> Self {
        match path {
            ChildPath::Leaf(leaf) => leaf,
            ChildPath::Path {
                entry,
                child,
                width,
                token_pos,
                ..
            } => PathLeaf {
                entry,
                child,
                width,
                token_pos,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChildPath {
    Leaf(PathLeaf),
    Path {
        entry: ChildLocation,
        path: Vec<ChildLocation>,
        child: Child,
        width: usize,
        token_pos: usize,
    },
}
impl ChildPath {
    //pub fn get_child(&self) -> Child {
    //    match self {
    //        Self::Leaf(leaf) => leaf.child,
    //        Self::Path { child, .. } => *child,
    //    }
    //}
    #[allow(unused)]
    pub fn get_leaf(&self) -> Option<&PathLeaf> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }
    #[allow(unused)]
    pub fn into_path(self) -> LocationPath {
        match self {
            Self::Leaf(_leaf) => Vec::new(),
            Self::Path { path, .. } => path,
        }
    }
    pub fn into_context_path(self) -> LocationPath {
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
    pub fn path(&self) -> &Vec<ChildLocation> {
        match self {
            ChildPath::Leaf(leaf) => leaf.path(),
            ChildPath::Path{ path, .. } => path.borrow(),
        }
    }
    pub fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        match self {
            ChildPath::Leaf(leaf) => leaf.path_mut(),
            ChildPath::Path{ path, .. } => path.borrow_mut(),
        }
    }
    pub fn num_path_segments(&self) -> usize {
        self.path().len() + 1
    }
    pub fn child_location(&self) -> ChildLocation {
        match self {
            ChildPath::Leaf(leaf) => leaf.child_location(),
            ChildPath::Path{ entry, .. } => *entry,
        }
    }
    pub fn child_location_mut(&mut self) -> &mut ChildLocation {
        match self {
            ChildPath::Leaf(leaf) => leaf.child_location_mut(),
            ChildPath::Path{ entry, .. } => &mut entry,
        }
    }
}
impl From<SearchPath> for ChildPath {
    fn from(p: SearchPath) -> Self {
        p.start
    }
}
impl From<PathLeaf> for ChildPath {
    fn from(leaf: PathLeaf) -> Self {
        ChildPath::Leaf(leaf)
    }
}
impl WideMut for ChildPath {
    fn width_mut(&mut self) -> &mut usize {
        match self {
            Self::Path { width, .. } => width,
            Self::Leaf(leaf) => leaf.width_mut(),
        }
    }
}

// todo: implement differently for end and start
//impl<D: MatchDirection> PathBorder<D> for ChildPath {
//    type BorderDirection = Back;
//}