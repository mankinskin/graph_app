use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EndPath {
    pub(crate) entry: ChildLocation,
    pub(crate) path: ChildPath,
}
impl<D: MatchDirection> DirectedBorderPath<D> for EndPath {
    type BorderDirection = Front;
}
impl GraphExit for EndPath {
    fn get_exit_location(&self) -> ChildLocation {
        self.entry
    }
}
impl HasEndPath for EndPath {
    fn end_path(&self) -> &[ChildLocation] {
        self.path()
    }
}
impl BorderPath for EndPath {
    fn path(&self) -> &[ChildLocation] {
        self.path.borrow()
    }
    fn entry(&self) -> ChildLocation {
        self.get_exit_location()
    }
}
impl ExitMut for EndPath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.entry.sub_index
    }
}