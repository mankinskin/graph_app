use crate::*;

pub trait IntoAdvanced<R: ResultKind>: Sized + Clone {
    fn into_advanced<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
    ) -> Result<R::Advanced, Self>;
    //{
    //    let mut new: R::Advanced = self.clone().into();
    //    match new.advance_exit_pos::<_, D, _>(trav) {
    //        Ok(()) => Ok(new),
    //        Err(()) => Err(self)
    //    }
    //}
}
//impl<
//    R: ResultKind,
//    T: Sized + Clone + Into<R::Advanced>
//> IntoAdvanced<R> for T {
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
impl IntoAdvanced<BaseResult> for RootedRolePath<Start> {
    fn into_advanced<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
    ) -> Result<<BaseResult as ResultKind>::Advanced, Self> {
        let entry = self.root_child_location();
        let graph = trav.graph();
        let pattern = self.root_pattern::<Trav>(&graph).clone();
        if let Some(next) = Trav::Direction::pattern_index_next(pattern.borrow(), entry.sub_index) {
            //let exit = entry.clone().to_child_location(next);
            //let child = pattern[next];
            Ok(SearchPath {
                root: entry.into_pattern_location(),
                end: RolePath {
                    sub_path: SubPath {
                        root_entry: next,
                        path: vec![],
                    }.into(),
                    //width: child.width(),
                    //token_pos: self.token_pos + child.width(),
                    //child,
                    _ty: Default::default(),
                },
                start: RolePath {
                    sub_path: self.split_path.sub_path,
                    _ty: Default::default(),
                },
            })
        } else {
            Err(self)
        }
    }
}
