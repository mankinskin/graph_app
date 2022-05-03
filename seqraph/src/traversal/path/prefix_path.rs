use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone)]
pub struct PrefixPath {
    pub(crate) pattern: Pattern,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}

impl<
    'a: 'g,
    'g,
> PrefixPath {
    pub fn new_directed<
        D: MatchDirection + 'a,
        P: IntoPattern,
    >(pattern: P) -> Result<Self, NoMatch> {
        let exit = D::head_index(pattern.borrow());
        let pattern = pattern.into_pattern();
        match pattern.len() {
            0 => Err(NoMatch::EmptyPatterns),
            1 => Err(NoMatch::SingleIndex),
            _ => 
            Ok(Self {
                pattern,
                exit,
                end: vec![],
            })
        }
    }
}
impl EntryPos for PrefixPath {
    fn get_entry_pos(&self) -> usize {
        0
    }
}
impl PatternEntry for PrefixPath {
    fn get_entry_pattern(&self) -> &[Child] {
        self.pattern.borrow()
    }
}
impl HasStartPath for PrefixPath {
    fn get_start_path(&self) -> &[ChildLocation] {
        &[]
    }
}
impl PatternStart for PrefixPath {}
impl EndPathMut for PrefixPath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
impl ExitPos for PrefixPath {
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
}
impl ExitMut for PrefixPath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl PatternExit for PrefixPath {
    fn get_exit_pattern(&self) -> &[Child] {
        &self.pattern
    }
}
impl HasEndPath for PrefixPath {
    fn get_end_path(&self) -> &[ChildLocation] {
        self.end.borrow()
    }
}
impl PatternEnd for PrefixPath {}
impl End for PrefixPath {
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
impl AdvanceablePath for PrefixPath {
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        if self.end.is_empty() {
            D::pattern_index_prev(self.pattern.borrow(), self.exit)
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
        D::pattern_index_next(self.pattern.borrow(), self.exit)
    }
}