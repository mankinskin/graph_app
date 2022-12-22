use crate::*;


pub trait NewAdvanced:
    Advance
    + GetCacheKey
{
    fn new_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
        A: Into<Self> + Clone + Send + Sync,
    >(
        trav: &'a Trav,
        start: A,
    ) -> Result<Self, A> {
        let mut new = start.clone().into();
        match new.advance_exit_pos::<_, D, _>(trav) {
            Ok(()) => Ok(new),
            Err(()) => Err(start)
        }
    }
}
impl<T:
    Advance
    + GetCacheKey
> NewAdvanced for T {
}

pub trait Advance:
    AdvanceExit
    + HasPath<End>
    + Descendant<End>
    + AdvanceWidth
    + Sized
{
    fn advance<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, trav: &'a Trav) -> Option<Child> {
        if self.is_finished(trav) {
            None
        } else {
            let current = self.get_descendant(trav);
            let graph = trav.graph();
            // skip path segments with no successors
            while let Some(mut location) = self.path_mut().pop() {
                let pattern = graph.expect_pattern_at(&location);
                if let Some(next) = D::pattern_index_next(pattern.borrow(), location.sub_index) {
                    location.sub_index = next;
                    //let next = pattern[next];
                    //self.advance_width(next.width);
                    self.path_mut().push(location);
                    return Some(current);
                }
            }
            // end is empty (exit is prev)
            let _ = self.advance_exit_pos::<_, D, _>(trav);
            Some(current)
        }
    }
}
impl<T: 
    AdvanceExit
    + AdvanceWidth
    + HasPath<End>
    + Descendant<End>
    + Sized
> Advance for T {
}

pub trait AdvanceWidth {
    fn advance_width(&mut self, width: usize);
}
impl <T: WideMut> AdvanceWidth for T {
    fn advance_width(&mut self, width: usize) {
        *self.width_mut() += width;
    }
}

pub trait AddMatchWidth: AdvanceWidth + Descendant<End> {
    fn add_match_width<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, trav: &'a Trav) {
        let end = self.get_descendant(trav);
        self.advance_width(end.width);
    }
}
impl<T: AdvanceWidth + Descendant<End>> AddMatchWidth for T {
}

pub trait AdvanceExit: ChildPosMut<End> + Send + Sync + Unpin {
    fn is_pattern_finished<
        P: IntoPattern,
    >(&self, pattern: P) -> bool {
        self.child_pos() >= pattern.borrow().len()
    }
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Result<Option<usize>, ()> {
        if self.is_pattern_finished(pattern.borrow()) {
            Err(())
        } else {
            Ok(D::pattern_index_next(pattern, self.child_pos()))
        }
    }
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> bool;
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> Result<Option<usize>, ()>;
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav)? {
            *self.child_pos_mut() = next;
            Ok(())
        } else {
            if !self.is_finished(trav) {
                *self.child_pos_mut() = D::index_next(self.child_pos()).expect("Can't represent behind end index!");
            }
            Err(())
        }
    }
}
pub trait FromAdvanced<A: Advanced> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: A, trav: &'a Trav) -> Self;
}
impl FromAdvanced<SearchPath> for FoundPath {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: SearchPath, trav: &'a Trav) -> Self {
        if path.is_complete::<_, D, _>(trav) {
            Self::Complete(<SearchPath as GraphChild<Start>>::child_location(&path).parent)
        } else {
            Self::Range(path)
        }
        
    }
}
impl FromAdvanced<OriginPath<SearchPath>> for OriginPath<FoundPath> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: OriginPath<SearchPath>, trav: &'a Trav) -> Self {
        Self {
            postfix: FoundPath::from_advanced::<_, D, _>(path.postfix, trav),
            origin: path.origin,
        }
    }
}

impl<A: Advanced, F: FromAdvanced<A>> FromAdvanced<A> for OriginPath<F> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: A, trav: &'a Trav) -> Self {
        Self {
            origin: MatchEnd::Path(HasRootedPath::<Start>::child_path(&path).clone()),
            postfix: F::from_advanced::<_, D, _>(path, trav),
        }
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
//            *self.child_pos_mut() = next;
//            Ok(())
//        } else {
//            self.context.advance_exit_pos::<_, D, _>(trav)
//        }
//    }
//}
impl<P: AdvanceExit> AdvanceExit for OriginPath<P> {
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> bool {
        self.postfix.is_finished(trav)
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, trav: &'a Trav) -> Result<Option<usize>, ()> {
        self.postfix.next_exit_pos::<_, D, _>(trav)
    }
}

impl<M:
    ChildPosMut<End>
    + PatternChild<End>
    + Send
    + Sync
    + Unpin
> AdvanceExit for M {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Result<Option<usize>, ()> {
        if self.is_pattern_finished(pattern.borrow()) {
            Err(())
        } else {
            Ok(D::pattern_index_next(pattern, self.child_pos()))
        }
    }
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> bool {
        self.is_pattern_finished(self.get_pattern())
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> Result<Option<usize>, ()> {
        self.pattern_next_exit_pos::<D, _>(self.get_pattern())
    }
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, _trav: &'a Trav) -> Result<(), ()> {
        let pattern = self.get_pattern();
        if let Some(next) = self.pattern_next_exit_pos::<D, _>(pattern.borrow())? {
            *self.child_pos_mut() = next;
            Ok(())
        } else {
            if !self.is_pattern_finished(pattern.borrow()) {
                *self.child_pos_mut() = D::index_next(self.child_pos()).expect("Can't represent behind end index!");
            }
            Err(())
        }
    }
}