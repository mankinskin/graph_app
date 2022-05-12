use super::*;

pub trait AdvanceablePath: EndPathMut + AdvanceableExit + End + PathFinished + Sized {
    fn try_advance<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Result<Self, Self> {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
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
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Self {
        self.try_advance::<_, D, _>(trav).unwrap_or_else(|e| e)
    }
    fn get_advance<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
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
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Result<(Child, Self), Child> {
        let current = self.get_end::<_, D, _>(trav);
        self.try_advance::<_, D, _>(trav)
            .map(|ad| (current, ad))
            .map_err(|_| current)
    }
}