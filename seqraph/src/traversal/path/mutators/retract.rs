use crate::*;

//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub struct RolePath {
//    pub entry: ChildLocation,
//    pub path: RolePath,
//    pub width: usize,
//}

pub trait Retract:
    RootPattern
    + PathPop
    + PathAppend
    + RootChildPosMut<End>
{
    fn prev_exit_pos<
        Trav: Traversable,
    >(&self, trav: &Trav) -> Option<usize> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<Trav>(&graph);
        <Trav::Kind as GraphKind>::Direction::pattern_index_prev(
            pattern.borrow(),
            self.root_child_pos()
        )
    }
    fn retract<
        Trav: Traversable,
        R: ResultKind,
    >(&mut self, trav: &Trav) {
        //let graph = trav.graph();
        //// remove segments pointing to mismatch at pattern head
        //while let Some(mut location) = self.path_mut().pop() {
        //    let pattern = graph.expect_pattern_at(&location);
        //    // skip segments at start of pattern
        //    if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
        //        location.sub_index = prev;
        //        self.path_mut().push(location);
        //        break;
        //    }
        //}
        //if self.path_mut().is_empty() {
        //    *self.root_child_pos_mut() = self.prev_exit_pos::<_, D, _>(trav).unwrap();
        //}
        let graph = trav.graph();
        // skip path segments with no successors
        if let Some(location) = std::iter::from_fn(|| 
            self.pop_path()
        ).find_map(|mut location| {
            let pattern = graph.expect_pattern_at(&location);
            <Trav::Kind as GraphKind>::Direction::pattern_index_prev(pattern.borrow(), location.sub_index)
                .map(|next| {
                    location.sub_index = next;
                    location
                })
        }) {
            self.path_append(trav, location);
        } else {
            *self.root_child_pos_mut() = self.prev_exit_pos(trav).unwrap();
        }

    }
}
impl<T:
    RootPattern
    + PathPop
    + PathAppend
    + RootChildPosMut<End>
> Retract for T
{
}