use super::*;

pub(crate) trait NewAdvanced: Advance {
    fn new_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
        A: Into<Self> + Clone,
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
impl<T: Advance> NewAdvanced for T {
}
pub(crate) trait Advance:
    EndPathMut
    + AdvanceExit
    + End
    + AdvanceWidth
    + Sized {
    fn advance<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> Option<Child> {
        if self.is_finished(trav) {
            None
        } else {
            let graph = trav.graph();
            let current = self.get_end::<_, D, _>(trav)?;
            // skip path segments with no successors
            while let Some(mut location) = self.end_path_mut().pop() {
                let pattern = graph.expect_pattern_at(&location);
                if let Some(next) = D::pattern_index_next(pattern.borrow(), location.sub_index) {
                    location.sub_index = next;
                    //let next = pattern[next];
                    //self.advance_width(next.width);
                    self.push_end(location);
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
    EndPathMut
    + AdvanceExit
    + End
    + AdvanceWidth
    + Sized
> Advance for T {
}
pub(crate) trait AdvanceWidth {
    fn advance_width(&mut self, width: usize);
}
impl <T: WideMut> AdvanceWidth for T {
    fn advance_width(&mut self, width: usize) {
        *self.width_mut() += width;
    }
}
pub(crate) trait AddMatchWidth: AdvanceWidth + End {
    fn add_match_width<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        if let Some(end) = self.get_end::<_, D, _>(trav) {
            self.advance_width(end.width);
        }
    }
}
impl<T: AdvanceWidth + End> AddMatchWidth for T {
}
pub(crate) trait AdvanceExit: ExitPos + ExitMut {
    fn is_pattern_finished<
        P: IntoPattern,
    >(&self, pattern: P) -> bool {
        self.get_exit_pos() >= pattern.borrow().len()
    }
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Result<Option<usize>, ()> {
        if self.is_pattern_finished(pattern.borrow()) {
            Err(())
        } else {
            Ok(D::pattern_index_next(pattern, self.get_exit_pos()))
        }
    }
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&self, _trav: &'a Trav) -> bool;
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&self, _trav: &'a Trav) -> Result<Option<usize>, ()>;
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav)? {
            *self.exit_mut() = next;
            Ok(())
        } else {
            if !self.is_finished(trav) {
                *self.exit_mut() = D::index_next(self.get_exit_pos()).expect("Can't represent behind end index!");
            }
            Err(())
        }
    }
}
impl<P: AdvanceExit> AdvanceExit for OriginPath<P> {
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&self, trav: &'a Trav) -> bool {
        self.postfix.is_finished(trav)
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&self, trav: &'a Trav) -> Result<Option<usize>, ()> {
        self.postfix.next_exit_pos::<_, D, _>(trav)
    }
}
impl<M: ExitMut
    + PatternExit
> AdvanceExit for M {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Result<Option<usize>, ()> {
        if self.is_pattern_finished(pattern.borrow()) {
            Err(())
        } else {
            Ok(D::pattern_index_next(pattern, self.get_exit_pos()))
        }
    }
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&self, _trav: &'a Trav) -> bool {
        self.is_pattern_finished(self.get_exit_pattern())
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&self, _trav: &'a Trav) -> Result<Option<usize>, ()> {
        self.pattern_next_exit_pos::<D, _>(self.get_exit_pattern())
    }
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T> + 'a,
    >(&mut self, _trav: &'a Trav) -> Result<(), ()> {
        let pattern = self.get_exit_pattern();
        if let Some(next) = self.pattern_next_exit_pos::<D, _>(pattern.borrow())? {
            *self.exit_mut() = next;
            Ok(())
        } else {
            if !self.is_pattern_finished(pattern.borrow()) {
                *self.exit_mut() = D::index_next(self.get_exit_pos()).expect("Can't represent behind end index!");
            }
            Err(())
        }
    }
}