use crate::{
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::IntoPatternLocation,
        },
        wide::Wide,
    },
    path::structs::{
        role_path::RolePath,
        rooted::{
            role_path::RootedRolePath,
            root::IndexRoot,
        },
        sub_path::SubPath,
    },
    traversal::{
        state::{
            cursor::RangeCursor,
            parent::ParentState,
        },
        traversable::Traversable,
    },
};

pub trait IntoPrimer: Sized {
    fn into_primer<Trav: Traversable>(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState;
}

impl IntoPrimer for (Child, RangeCursor) {
    fn into_primer<Trav: Traversable>(
        self,
        _trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState {
        let (c, cursor) = self;
        let width = c.width().into();
        ParentState {
            prev_pos: width,
            root_pos: width,
            path: RootedRolePath {
                root: IndexRoot {
                    location: parent_entry.into_pattern_location(),
                },
                role_path: RolePath {
                    sub_path: SubPath {
                        root_entry: parent_entry.sub_index,
                        path: vec![],
                    },
                    _ty: Default::default(),
                },
            },
            cursor,
        }
    }
}
