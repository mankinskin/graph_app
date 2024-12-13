use crate::{
    path::{
        accessors::role::End,
        structs::{
            query_range_path::PatternPrefixPath,
            role_path::RolePath,
        },
    },
    graph::vertex::child::Child,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlapPrimer {
    pub start: Child,
    pub context: PatternPrefixPath,
    //pub context_offset: usize,
    pub width: usize,
    pub exit: usize,
    pub end: RolePath<End>,
}

impl OverlapPrimer {
    pub fn new(
        start: Child,
        context: PatternPrefixPath,
    ) -> Self {
        Self {
            start,
            //context_offset: context.root_child_pos(),
            context,
            width: 0,
            exit: 0,
            end: RolePath::default(),
        }
    }
    pub fn into_prefix_path(self) -> PatternPrefixPath {
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
//            1 => if self.context.root_child_pos() > self.context_offset {
//                self.context.prev_exit_pos::<_, D, _>(trav)
//            } else {
//                Some(0)
//            },
//            _ => Some(1),
//        }
//    }
//}
