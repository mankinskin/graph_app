use crate::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct OverlapPrimer {
    pub start: Child,
    pub context: PrefixQuery,
    pub context_offset: usize,
    pub width: usize,
    pub exit: usize,
    pub end: LocationPath,
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

//impl TraversalPath for OverlapPrimer {
//    fn prev_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: Trav) -> Option<usize> {
//        match self.exit {
//            0 => None,
//            1 => if self.context.child_pos() > self.context_offset {
//                self.context.prev_exit_pos::<_, D, _>(trav)
//            } else {
//                Some(0)
//            },
//            _ => Some(1),
//        }
//    }
//}