use std::ops::ControlFlow;

use crate::structs::role_path::{RolePath};
use crate::{
    path::structs::sub_path::SubPath,
    trace::has_graph::HasGraph,
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
    fn path_lower<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()>;
}
