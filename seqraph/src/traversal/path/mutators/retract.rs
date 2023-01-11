use crate::*;

//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub struct ChildPath {
//    pub entry: ChildLocation,
//    pub path: ChildPath,
//    pub width: usize,
//}

pub trait Retract:
    RootPattern
    + PathChild<End>
    + HasRolePath<End>
    + RootChildPosMut<End>
    + Send
    + Sync
{
    fn prev_exit_pos<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Option<usize> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<_, Trav>(&graph);
        D::pattern_index_prev(
            pattern.borrow(),
            self.root_child_pos()
        )
    }
    fn retract<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        R: ResultKind,
    >(&mut self, trav: &Trav) {
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
            *self.root_child_pos_mut() = self.prev_exit_pos::<_, D, _>(trav).unwrap();
        }

    }
}
impl<T:
    RootPattern
    + PathChild<End>
    + HasRolePath<End>
    + RootChildPosMut<End>
    + Send
    + Sync
> Retract for T
{
}