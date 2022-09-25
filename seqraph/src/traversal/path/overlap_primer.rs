use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct OverlapPrimer {
    pub(crate) start: Child,
    pub(crate) context: PrefixQuery,
    pub(crate) context_offset: usize,
    pub(crate) width: usize,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl OverlapPrimer {
    pub fn new(start: Child, context: PrefixQuery) -> Self {
        Self {
            start,
            context_offset: context.exit,
            context,
            width: 0,
            exit: 0,
            end: vec![],
        }
    }
    pub fn into_prefix_path(self) -> PrefixQuery {
        self.context
    }
}
impl ExitPos for OverlapPrimer {
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
}
impl HasEndPath for OverlapPrimer {
    //type Path = [ChildLocation];
    fn end_path(&self) -> &[ChildLocation] {
        if self.exit == 0 {
            self.end.borrow()
        } else {
            self.context.end.borrow()
        }
    }
}
impl End for OverlapPrimer {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        if self.exit == 0 {
            Some(self.start)
        } else {
            self.context.get_pattern_end(trav)
        }
    }
}
impl EntryPos for OverlapPrimer {
    fn get_entry_pos(&self) -> usize {
        0
    }
}
//impl TraversalPath for OverlapPrimer {
//    fn prev_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<'a, 'g, T>,
//    >(&self, trav: &'a Trav) -> Option<usize> {
//        match self.exit {
//            0 => None,
//            1 => if self.context.get_exit_pos() > self.context_offset {
//                self.context.prev_exit_pos::<_, D, _>(trav)
//            } else {
//                Some(0)
//            },
//            _ => Some(1),
//        }
//    }
//}
impl AdvanceExit for OverlapPrimer {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, _pattern: P) -> Result<Option<usize>, ()> {
        Ok(None)
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Result<Option<usize>, ()> {
        Ok(if self.exit == 0 {
            Some(1)
        } else {
            None
        })
    }
    fn is_finished<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        self.context.is_finished(trav)
    }
    fn advance_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> Result<(), ()> {
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav)? {
            *self.exit_mut() = next;
            Ok(())
        } else {
            self.context.advance_exit_pos::<_, D, _>(trav)
        }
    }
}
impl Wide for OverlapPrimer {
    fn width(&self) -> usize {
        self.width
    }
}
impl WideMut for OverlapPrimer {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}