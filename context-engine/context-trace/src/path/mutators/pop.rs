use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::role::PathRole,
        structs::{
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::PathRoot,
            },
        },
    },
};

// pop path segments
pub trait PathPop {
    fn path_pop(&mut self) -> Option<ChildLocation>;
}

impl<Role: PathRole, Root: PathRoot> PathPop for RootedRolePath<Role, Root> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.role_path.path_pop()
    }
}

impl<R: PathRole> PathPop for RolePath<R> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.sub_path.path.pop()
    }
}
