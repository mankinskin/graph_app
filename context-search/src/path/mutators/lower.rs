use std::ops::ControlFlow;

use super::super::{
    accessors::role::End,
    structs::role_path::RolePath,
};
use crate::{
    path::structs::sub_path::SubPath,
    traversal::traversable::Traversable,
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
