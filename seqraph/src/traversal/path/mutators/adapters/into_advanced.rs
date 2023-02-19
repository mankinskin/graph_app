use crate::*;

pub trait IntoAdvanced: Sized + Clone {
    fn into_advanced<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
    ) -> Result<SearchPath, Self>;
    //{
    //    let mut new: SearchPath = self.clone().into();
    //    match new.advance_exit_pos::<_, D, _>(trav) {
    //        Ok(()) => Ok(new),
    //        Err(()) => Err(self)
    //    }
    //}
}
//impl<
//    R: ResultKind,
//    T: Sized + Clone + Into<SearchPath>
//> IntoAdvanced for T {
//}
//impl<
//    P: IntoAdvanced<BaseResult>,
//> IntoAdvanced<OriginPathResult> for OriginPath<P> {
//    fn into_advanced<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(
//        self,
//        trav: &Trav,
//    ) -> Result<<OriginPathResult as ResultKind>::Advanced, Self> {
//        match self.postfix.into_advanced::<_, D, _>(trav) {
//            Ok(path) => Ok(OriginPath {
//                postfix: path,
//                origin: self.origin,
//            }),
//            Err(path) => Err(OriginPath {
//                postfix: path,
//                origin: self.origin,
//            }),
//        }
//    }
//}
impl IntoAdvanced for RootedRolePath<Start> {
    fn into_advanced<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
    ) -> Result<SearchPath, Self> {
        let entry = self.root_child_location();
        let graph = trav.graph();
        let pattern = self.root_pattern::<Trav>(&graph).clone();
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), entry.sub_index) {
            Ok(SearchPath {
                root: IndexRoot {
                    location: entry.into_pattern_location(),
                    pos: self.split_path.root.pos,
                },
                start: RolePath {
                    sub_path: self.split_path.sub_path,
                    _ty: Default::default(),
                },
                end: RolePath {
                    sub_path: SubPath {
                        root_entry: next,
                        path: vec![],
                    }.into(),
                    _ty: Default::default(),
                },
            })
        } else {
            Err(self)
        }
    }
}
