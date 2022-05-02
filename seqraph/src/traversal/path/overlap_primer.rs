use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone)]
pub struct OverlapPrimer {
    start: Child,
    context: Pattern,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl OverlapPrimer {
    pub fn new(start: Child, context: PrefixPath) -> Self {
        Self {
            start,
            context: context.pattern,
            exit: context.exit,
            end: context.end,
        }
    }
}
impl EntryPos for OverlapPrimer {
    fn get_entry_pos(&self) -> usize {
        0
    }
}
impl PatternEntry for OverlapPrimer {
    fn get_entry_pattern(&self) -> &[Child] {
        self.start.borrow()
    }
}
impl HasStartPath for OverlapPrimer {
    fn get_start_path(&self) -> &[ChildLocation] {
        &[]
    }
}
impl PatternStart for OverlapPrimer {}
impl ExitPos for OverlapPrimer {
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
}
impl PatternExit for OverlapPrimer {
    fn get_exit_pattern(&self) -> &[Child] {
        &self.context
    }
}
impl HasEndPath for OverlapPrimer {
    fn get_end_path(&self) -> &[ChildLocation] {
        self.end.borrow()
    }
}
impl PatternEnd for OverlapPrimer {}
impl RangePath for OverlapPrimer {
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end.push(location);
                break;
            }
        }
        if self.end.is_empty() {
            self.exit = self.prev_pos::<_, D, _>(trav).unwrap();
        }

        self
    }
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        if self.end.is_empty() {
            D::pattern_index_prev(self.context.borrow(), self.exit)
        } else {
            let location = self.end.last().unwrap().clone();
            let pattern = trav.graph().expect_pattern_at(&location);
            D::pattern_index_prev(pattern, location.sub_index)
        }
    }
    fn advance_next<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = D::pattern_index_next(self.context.borrow(), self.exit) {
            self.exit = next;
            true
        } else {
            false
        }
    }
}