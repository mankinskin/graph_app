use super::*;

pub(crate) trait AdvanceablePath:
    EndPathMut
    + AdvanceableExit
    + End
    + PathFinished
    + AdvanceableWidth
    + Sized {
    fn try_advance<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Result<Self, Self> {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            if let Some(next) = D::pattern_index_next(pattern.borrow(), location.sub_index) {
                self.advance_width(pattern[next].width);
                location.sub_index = next;
                self.push_end(location);
                return Ok(self);
            }
        }
        // end is empty (exit is prev)
        match self.advance_exit_pos::<_, D, _>(trav) {
            Ok(()) => Ok(self),
            Err(()) => Err(self)
        }
    }
    fn into_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Self {
        self.try_advance::<_, D, _>(trav).unwrap_or_else(|e| e)
    }
    fn get_advance<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> (Child, Self) {
        (
            self.get_end::<_, D, _>(trav),
            self.into_advanced::<_, D, _>(trav),
        )
    }
    fn try_get_advance<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Result<(Child, Self), Child> {
        let current = self.get_end::<_, D, _>(trav);
        self.try_advance::<_, D, _>(trav)
            .map(|ad| (current, ad))
            .map_err(|_| current)
    }
}
pub(crate) trait AdvanceableWidth {
    fn advance_width(&mut self, width: usize);
}
impl <T: WideMut> AdvanceableWidth for T {
    fn advance_width(&mut self, width: usize) {
        *self.width_mut() += width;
    }
}
pub(crate) trait AdvanceableExit: ExitPos + ExitMut + PathFinished {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Option<usize>;
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize>;
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav) {
            *self.exit_mut() = next;
            Ok(())
        } else {
            self.set_finished();
            Err(())
        }
    }
}
impl<M:
    ExitMut
    + PatternExit
    + PathFinished
    //+ HasInnerWidth
    //+ HasEndWidth
> AdvanceableExit for M {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, pattern: P) -> Option<usize> {
        D::pattern_index_next(pattern, self.get_exit_pos())
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize> {
        self.pattern_next_exit_pos::<D, _>(self.get_exit_pattern())
    }
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, _trav: &'a Trav) -> Result<(), ()> {
        let pattern = self.get_exit_pattern();
        if let Some(next) = self.pattern_next_exit_pos::<D, _>(pattern.borrow()) {
            *self.exit_mut() = next;
            Ok(())
        } else {
            self.set_finished();
            Err(())
        }
    }
}