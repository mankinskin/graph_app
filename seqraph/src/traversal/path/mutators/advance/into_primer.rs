use crate::*;

pub trait IntoPrimer<R: ResultKind>: Sized {
    fn into_primer<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> R::Primer;
}
impl<R: ResultKind> IntoPrimer<R> for MatchEnd<RootedRolePath<Start>>{
    fn into_primer<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> R::Primer {
        R::Primer::from(match self {
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
                path.path_append(trav, parent_entry);
                path
            },
        })
    }
}