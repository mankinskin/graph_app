use crate::*;

pub trait IntoAdvanced: Sized + Clone {
    fn into_advanced<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
    ) -> Result<ChildState, Self>;
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
impl IntoAdvanced for ParentState {
    fn into_advanced<
        Trav: Traversable,
    >(
        self,
        trav: &Trav,
    ) -> Result<ChildState, Self> {
        let entry = self.path.root_child_location();
        let graph = trav.graph();
        let pattern = self.path.root_pattern::<Trav>(&graph).clone();
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), entry.sub_index) {
            let index = pattern[next];
            Ok(
                ChildState {
                    prev_pos: self.prev_pos,
                    root_pos: self.root_pos,
                    matched: self.matched,
                    paths: PathPair::new(
                        SearchPath {
                            root: self.path.split_path.root,
                            start: RolePath {
                                sub_path: self.path.split_path.sub_path,
                                _ty: Default::default(),
                            },
                            end: RolePath {
                                sub_path: SubPath {
                                    root_entry: next,
                                    path: vec![],
                                }.into(),
                                _ty: Default::default(),
                            },
                        },
                        self.query,
                        PathPairMode::GraphMajor,
                    ),
                    target: CacheKey::new(
                        index,
                        self.root_pos + index.width(),
                    )
                }
            )
        } else {
            Err(self)
        }
    }
}
