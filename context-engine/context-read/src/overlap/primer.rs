use context_trace::{
    graph::vertex::child::Child,
    path::{
        accessors::role::End,
        structs::{
            role_path::RolePath,
            rooted::role_path::PatternEndPath,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlapPrimer {
    /// postfix of the previous block
    pub postfix: Child,
    /// the path up until postfix
    pub context: PatternEndPath,
    //pub context_offset: usize,
    pub width: usize,
    pub exit: usize,
    pub end: RolePath<End>,
}

impl OverlapPrimer {
    pub fn new(
        postfix: Child,
        context: PatternEndPath,
    ) -> Self {
        Self {
            postfix,
            //context_offset: context.root_child_index(),
            context,
            width: 0,
            exit: 0,
            end: RolePath::default(),
        }
    }
    pub fn into_prefix_path(self) -> PatternEndPath {
        self.context
    }
    //pub fn into_query_state(self) -> QueryState {
    //}
}
//impl Foldable for OverlapPrimer {
//    fn fold<'a, K: TraversalKind>(self, trav: &'a K::G) -> FoldResult {
//        FoldContext::<K>::fold_query(trav, self)
//    }
//}

//impl TraversalPath for OverlapPrimer {
//    fn prev_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        G: HasGraph<T>,
//    >(&self, trav: G) -> Option<usize> {
//        match self.exit {
//            0 => None,
//            1 => if self.context.root_child_index() > self.context_offset {
//                self.context.prev_exit_pos::<_, D, _>(trav)
//            } else {
//                Some(0)
//            },
//            _ => Some(1),
//        }
//    }
//}
