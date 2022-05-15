use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EndPath {
    pub(crate) entry: ChildLocation,
    pub(crate) path: ChildPath,
    pub(crate) width: usize,
}
impl<D: MatchDirection> DirectedBorderPath<D> for EndPath {
    type BorderDirection = Front<D>;
}
impl Wide for EndPath {
    fn width(&self) -> usize {
        self.width
    }
}
impl GraphExit for EndPath {
    fn get_exit_location(&self) -> ChildLocation {
        self.entry
    }
}
impl HasEndPath for EndPath {
    fn get_end_path(&self) -> &[ChildLocation] {
        self.path.as_slice()
    }
}
impl BorderPath for EndPath {
    fn path(&self) -> &[ChildLocation] {
        self.get_end_path()
    }
    fn entry(&self) -> ChildLocation {
        self.get_exit_location()
    }
}