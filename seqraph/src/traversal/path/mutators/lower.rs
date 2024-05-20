use std::ops::ControlFlow;

use crate::{
    traversal::{
        path::{
            accessors::role::End,
            mutators::move_path::{
                key::TokenLocation,
                RetractKey,
            },
            structs::{
                role_path::RolePath,
                rooted_path::{
                    SearchPath,
                    SubPath,
                },
            },
        },
        traversable::Traversable,
    },
    vertex::{
        location::IntoPatternLocation,
        wide::Wide,
    },
};

pub trait PathLower {
    fn end_path(index: usize) -> RolePath<End> {
        RolePath {
            sub_path: SubPath {
                root_entry: index,
                path: vec![],
            },
            _ty: Default::default(),
        }
    }
    fn path_lower<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()>;
}
impl PathLower for (&mut TokenLocation, &mut SearchPath) {
    fn path_lower<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let (root_pos, range) = self;
        let (start, end, root) = (
            &mut range.start.sub_path,
            &mut range.end.sub_path,
            &mut range.root,
        );
        if let Some(prev) = start.path.pop() {
            let graph = trav.graph();
            let pattern = graph.expect_pattern_at(&prev);
            root_pos.retract_key(
                pattern[prev.sub_index + 1..]
                    .iter()
                    .fold(0, |a, c| a + c.width()),
            );
            start.root_entry = prev.sub_index;
            end.root_entry = pattern.len() - 1;
            end.path.clear();
            root.location = prev.into_pattern_location();

            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
