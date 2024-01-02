use crate::shared::*;

pub trait IntoPrimer: Sized {
    fn into_primer<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState;
}
impl IntoPrimer for (Child, QueryState) {
    fn into_primer<
        Trav: Traversable,
    >(
        self,
        _trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState {
        let (c, query) = self;
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
            query,
        }
    }
}