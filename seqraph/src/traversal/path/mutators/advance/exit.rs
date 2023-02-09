use crate::*;

pub trait AdvanceRootPos<R: PathRole> {
    fn advance_root_pos<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()>;
}
//pub trait AdvanceExit {
//    fn next_exit_pos<
//        Trav: Traversable,
//    >(&self, _trav: &Trav) -> Option<usize>;
//
//    fn pattern_next_exit_pos<
//        D: MatchDirection,
//        P: IntoPattern,
//    >(&self, pattern: P) -> Option<usize>;
//
//    fn advance_exit_pos<
//        Trav: Traversable,
//    >(&mut self, trav: &Trav) -> ControlFlow<()>;
//}

impl AdvanceRootPos<End> for SearchPath {
    //fn next_exit_pos<
    //    Trav: Traversable,
    //>(&self, trav: &Trav) -> Option<usize> {
    //    let location = self.root_pattern_location();
    //    let graph = trav.graph();
    //    let pattern = graph.expect_pattern_at(&location);
    //    self.pattern_next_exit_pos::<TravDir<Trav>, _>(pattern.borrow())
    //}
    //fn pattern_next_exit_pos<
    //    D: MatchDirection,
    //    P: IntoPattern,
    //>(&self, pattern: P) -> Option<usize> {
    //    D::pattern_index_next(pattern, RootChildPos::<End>::root_child_pos(self))
    //}
    fn advance_root_pos<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<Trav>(&graph);
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), RootChildPos::<End>::root_child_pos(self)) {
            *self.root_child_pos_mut() = next;
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}
impl AdvanceRootPos<End> for CachedQuery<'_> {
    fn advance_root_pos<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        let pattern = &self.cache.query_root;
        let exit = self.state.end.root_child_pos_mut();
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), *exit) {
            *exit = next;
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}
impl AdvanceRootPos<End> for QueryRangePath {
    fn advance_root_pos<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        if let Some(next) = TravDir::<Trav>::index_next(RootChildPos::<End>::root_child_pos(self)) {
            *self.root_child_pos_mut() = next;
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}
//impl<M:
//    RootChildPosMut<End>
//    + PatternRootChild<End>
//> AdvanceExit for M {
//    fn next_exit_pos<
//        Trav: Traversable,
//    >(&self, _trav: &Trav) -> Option<usize> {
//        self.pattern_next_exit_pos::<TravDir<Trav>, _>(self.pattern_root_pattern().borrow())
//    }
//    fn pattern_next_exit_pos<
//        D: MatchDirection,
//        P: IntoPattern,
//    >(&self, pattern: P) -> Option<usize> {
//        D::pattern_index_next(pattern, self.root_child_pos())
//    }
//    fn advance_exit_pos<
//        Trav: Traversable,
//    >(&mut self, trav: &Trav) -> ControlFlow<()> {
//        if let Some(next) = self.next_exit_pos(trav) {
//            *self.root_child_pos_mut() = next;
//            ControlFlow::CONTINUE
//        } else {
//            *self.root_child_pos_mut() = TravDir::<Trav>::index_next(self.root_child_pos()).expect("Can't represent behind end index!");
//            ControlFlow::BREAK
//        }
//    }
//}
//impl<P: AdvanceExit> AdvanceExit for OriginPath<P> {
//    fn is_finished<
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &Trav) -> bool {
//        self.postfix.is_finished(trav)
//    }
//    fn next_exit_pos<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: &Trav) -> Result<Option<usize>, ()> {
//        self.postfix.next_exit_pos::<_, D, _>(trav)
//    }
//}
//impl AdvanceExit for OverlapPrimer {
//    fn pattern_next_exit_pos<
//        D: MatchDirection,
//        P: IntoPattern,
//    >(&self, _pattern: P) -> Result<Option<usize>, ()> {
//        Ok(None)
//    }
//    fn next_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, _trav: &'a Trav) -> Result<Option<usize>, ()> {
//        Ok(if self.exit == 0 {
//            Some(1)
//        } else {
//            None
//        })
//    }
//    fn is_finished<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> bool {
//        self.context.is_finished(trav)
//    }
//    fn advance_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
//        if let Some(next) = self.next_exit_pos::<_, D, _>(trav)? {
//            *self.root_child_pos_mut() = next;
//            Ok(())
//        } else {
//            self.context.advance_exit_pos::<_, D, _>(trav)
//        }
//    }
//}