use crate::*;

pub trait AdvanceExit:
    RootChildPosMut<End>
    + BasePath
 {
    //fn is_pattern_finished<
    //    P: IntoPattern,
    //>(&self, pattern: P) -> bool {
    //    self.root_child_pos() >= pattern.borrow().len()
    //}
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Option<usize> {
        //if self.is_pattern_finished(pattern.borrow()) {
        //    Err(())
        //} else {
        //    Ok()
        //}
        D::pattern_index_next(pattern, self.root_child_pos())
    }

    //fn is_finished<
    //    T: Tokenize,
    //    Trav: Traversable<T>,
    //>(&self, _trav: &Trav) -> bool;

    fn next_exit_pos<
        Trav: Traversable,
    >(&self, _trav: &Trav) -> Option<usize>;

    fn advance_exit_pos<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        if let Some(next) = self.next_exit_pos(trav) {
            *self.root_child_pos_mut() = next;
            ControlFlow::CONTINUE
        } else {
            //if !self.is_finished(trav) {
            //}
            *self.root_child_pos_mut() = Trav::Direction::index_next(self.root_child_pos()).expect("Can't represent behind end index!");
            ControlFlow::BREAK
        }
    }
}
impl<M:
    RootChildPosMut<End>
    + PatternRootChild<End>
    + BasePath
> AdvanceExit for M {
    //fn is_finished<
    //    T: Tokenize,
    //    Trav: Traversable<T>,
    //>(&self, _trav: &Trav) -> bool {
    //    self.is_pattern_finished(self.pattern_root_pattern().borrow())
    //}
    fn next_exit_pos<
        Trav: Traversable,
    >(&self, _trav: &Trav) -> Option<usize> {
        self.pattern_next_exit_pos::<Trav::Direction, _>(self.pattern_root_pattern().borrow())
    }
}
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

impl AdvanceExit for SearchPath {
    //fn is_finished<
    //    T: Tokenize,
    //    Trav: Traversable<T>,
    //>(&self, trav: &Trav) -> bool {
    //    let location = <Self as GraphRootChild<End>>::root_child_location(self);
    //    let graph = trav.graph();
    //    let pattern = graph.expect_pattern_at(&location);
    //    self.is_pattern_finished(pattern.borrow())
    //}
    fn next_exit_pos<
        Trav: Traversable,
    >(&self, trav: &Trav) -> Option<usize> {
        let location = self.root_pattern_location();
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&location);
        self.pattern_next_exit_pos::<Trav::Direction, _>(pattern.borrow())
    }
}
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