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
impl EndPathMut for OverlapPrimer {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
impl ExitMut for OverlapPrimer {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl End for OverlapPrimer {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        self.get_pattern_end::<_, D, _>(trav)
    }
}
impl AdvanceablePath for OverlapPrimer {
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
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        D::pattern_index_next(self.context.borrow(), self.exit)
    }
}