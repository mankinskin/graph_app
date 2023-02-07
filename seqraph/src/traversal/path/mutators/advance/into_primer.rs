use crate::*;

pub trait IntoPrimer: Sized {
    fn into_primer<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> Primer;
}
impl IntoPrimer for MatchEnd<RootedRolePath<Start>>{
    fn into_primer<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> Primer {
        Primer::from(match self {
            Self::Complete(_) => RootedRolePath {
                split_path: RootedSplitPath {
                    sub_path: SubPath {
                        root_entry: parent_entry.sub_index,
                        path: vec![],
                    },
                    root: parent_entry.into_pattern_location(),
                },
                _ty: Default::default(),
            },
            Self::Path(mut path) => {
                path.path_raise(trav, parent_entry);
                path
            },
        })
    }
}