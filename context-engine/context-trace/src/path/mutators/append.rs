use super::super::{
    accessors::role::{
        End,
        PathRole,
    },
    structs::role_path::RolePath,
};
use crate::{
    graph::vertex::location::child::ChildLocation,
    path::structs::{
        rooted::{
            index_range::IndexRangePath,
            pattern_range::PatternRangePath,
            role_path::RootedRolePath,
            root::PathRoot,
        },
        sub_path::SubPath,
    },
};

/// move path leaf position one level deeper
pub trait PathAppend {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    );
}

impl<Role: PathRole, Root: PathRoot> PathAppend for RootedRolePath<Role, Root> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.role_path.sub_path.path_append(parent_entry);
    }
}

impl PathAppend for SubPath {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.path.push(parent_entry)
    }
}

impl PathAppend for RolePath<End> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.sub_path.path.push(parent_entry)
    }
}

impl PathAppend for IndexRangePath {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.end.sub_path.path.push(parent_entry);
    }
}

impl PathAppend for PatternRangePath {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.end.sub_path.path.push(parent_entry);
    }
}
