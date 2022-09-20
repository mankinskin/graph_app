use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EndPath {
    pub(crate) entry: ChildLocation,
    pub(crate) path: ChildPath,
    pub(crate) width: usize,
}
impl PathReduce for EndPath {
    fn reduce<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.path.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.path.push(location);
                break;
            }
        }
    }
}
//impl BorderPath for EndPath {
//    fn path(&self) -> &[ChildLocation] {
//        self.path.borrow()
//    }
//    fn entry(&self) -> ChildLocation {
//        self.get_exit_location()
//    }
//}
impl<D: MatchDirection> PathBorder<D> for EndPath {
    type BorderDirection = Front;
}
impl HasSinglePath for EndPath {
    fn single_path(&self) -> &[ChildLocation] {
        self.end_path()
    }
}
impl PathRoot for EndPath {
    fn root(&self) -> ChildLocation {
        self.get_exit_location()
    }
}
impl GraphExit for EndPath {
    fn get_exit_location(&self) -> ChildLocation {
        self.entry
    }
}
impl HasEndPath for EndPath {
    fn end_path(&self) -> &[ChildLocation] {
        self.path.borrow()
    }
}
impl ExitMut for EndPath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.entry.sub_index
    }
}
impl WideMut for EndPath {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}
impl Wide for EndPath {
    fn width(&self) -> usize {
        self.width
    }
}