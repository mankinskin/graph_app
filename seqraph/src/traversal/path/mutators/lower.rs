
use crate::*;

pub trait PathLower {
    fn end_path(index: usize) -> RolePath<End> {
        RolePath {
            sub_path: SubPath {
                root_entry: index,
                path: vec![]
            },
            _ty: Default::default(),
        }
    }
    fn path_lower<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav
    ) -> ControlFlow<()>;
}
impl PathLower for (&mut TokenLocation, &mut RangeEnd) {
    fn path_lower<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let (root_pos, range) = self;
        let (start, root) = (&mut range.path.start.sub_path, &mut range.path.root);
        if let Some(prev) = start.path.pop() {
            // pop root
            //let end = Self::end_path(
            //    <Trav::Kind as GraphKind>::Direction::last_index(
            //        trav.graph().expect_pattern_at(&prev).borrow()
            //    )
            //);
            let graph = trav.graph();
            let pattern = graph.expect_pattern_at(&prev);
            root_pos.retract_key(
                pattern[prev.sub_index+1..].iter().fold(0, |a, c| a + c.width())
            );
            start.root_entry = prev.sub_index;
            root.location = prev.into_pattern_location();
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}