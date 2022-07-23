use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct OverlapPrimer {
    pub(crate) start: Child,
    context: PrefixPath,
    context_offset: usize,
    width: usize,
    exit: usize,
    end: ChildPath,
}
impl OverlapPrimer {
    pub fn new(start: Child, context: PrefixPath) -> Self {
        Self {
            start,
            context_offset: context.exit,
            context,
            width: 0,
            exit: 0,
            end: vec![],
        }
    }
    pub fn into_prefix_path(self) -> PrefixPath {
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
impl EndPathMut for OverlapPrimer {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        if self.exit == 0 {
            &mut self.end
        } else {
            &mut self.context.end
        }
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
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if self.exit == 0 {
            self.start
        } else {
            self.context.get_pattern_end(trav)
        }
    }
}
impl PathFinished for OverlapPrimer {
    fn is_finished(&self) -> bool {
        self.context.finished
    }
    fn set_finished(&mut self) {
        self.context.finished = true;
    }
}
impl ReduciblePath for OverlapPrimer {
    fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        match self.exit {
            0 => None,
            1 => if self.context.get_exit_pos() > self.context_offset {
                self.context.prev_exit_pos::<_, D, _>(trav)
            } else {
                Some(0)
            },
            _ => Some(1),
        }
    }
}
impl AdvanceableExit for OverlapPrimer {
    fn pattern_next_exit_pos<
        D: MatchDirection,
        P: IntoPattern,
    >(&self, _pattern: P) -> Option<usize> {
        None
    }
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, _trav: &'a Trav) -> Option<usize> {
        if self.exit == 0 {
            Some(1)
        } else {
            None
        }
    }
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
//impl HasEndWidth for OverlapPrimer {
//    fn end_width(&self) -> usize {
//        self.context.end_width()
//    }
//    fn end_width_mut(&mut self) -> &mut usize {
//        self.context.end_width_mut()
//    }
//}
impl AdvanceablePath for OverlapPrimer {}