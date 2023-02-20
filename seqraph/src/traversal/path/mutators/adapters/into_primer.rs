use crate::*;

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
                split_path: RootedSplitPath {
                    sub_path: SubPath {
                        root_entry: parent_entry.sub_index,
                        path: vec![],
                    },
                    root: IndexRoot {
                        location: parent_entry.into_pattern_location(),
                    },
                },
                _ty: Default::default(),
            },
            query,
        }
    }
}