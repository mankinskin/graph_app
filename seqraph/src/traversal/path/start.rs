use super::*;
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
impl From<GraphRangePath> for StartPath {
    fn from(p: GraphRangePath) -> Self {
        p.start
    }
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
    fn path(&self) -> &[ChildLocation] {
        self.get_start_path()
    }
    fn entry(&self) -> ChildLocation {
        self.get_entry_location()
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
impl GraphEntry for StartPath {
    fn get_entry_location(&self) -> ChildLocation {
        match self {
            Self::Path { entry, .. } |
            Self::First { entry, .. }
                => *entry,
        }
    }
}
impl HasStartPath for StartPath {
    fn get_start_path(&self) -> &[ChildLocation] {
        match self {
            Self::Path { path, .. } => path.as_slice(),
            _ => &[],
        }
    }
}