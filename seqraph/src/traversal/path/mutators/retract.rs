use crate::*;

//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub struct ChildPath {
//    pub entry: ChildLocation,
//    pub path: ChildPath,
//    pub width: usize,
//}

pub trait Retract:
    Root
    + Descendant<End>
    + HasRootedPath<End>
    + ChildPosMut<End>
    + Send
    + Sync
{
    fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        D::pattern_index_prev(self.pattern(trav), self.child_pos())
    }
    fn retract<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        R: ResultKind,
    >(&mut self, trav: &'a Trav) {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at start of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.path_mut().push(location);
                break;
            }
        }
        if self.path_mut().is_empty() {
            *self.child_pos_mut() = self.prev_exit_pos::<_, D, _>(trav).unwrap();
        }

    }
}
impl<T:
    Root
    + Descendant<End>
    + HasRootedPath<End>
    + ChildPosMut<End>
    + Send
    + Sync
> Retract for T
{
}
//impl GraphChild for ChildPath {
//    fn entry(&self) -> ChildLocation {
//        self.entry
//    }
//}
//impl BorderPath for ChildPath {
//    fn path(&self) -> &[ChildLocation] {
//        self.path.borrow()
//    }
//    fn entry(&self) -> ChildLocation {
//        self.child_location()
//    }
//}
//impl<D: MatchDirection> PathBorder<D> for ChildPath {
//    type BorderDirection = Front;
//}