use crate::{
    direction::{
        r#match::MatchDirection,
        Right,
    },
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            pattern::IntoPattern,
        },
    },
    path::{
        accessors::{
            child::LeafChild,
            role::End,
        },
        mutators::{
            append::PathAppend,
            move_path::root::MoveRootPos,
            pop::PathPop,
        },
        structs::rooted::root::RootedPath,
        BaseQuery,
    },
    traversal::{
        result::kind::RoleChildPath,
        traversable::Traversable,
    },
    //traversal::state::query::QueryState
};

use super::rooted::pattern_range::PatternRangePath;

pub trait FoldablePath:
BaseQuery
//+ LeafChildPosMut<End>
+ PathAppend
+ PathPop
+ MoveRootPos<Right, End>
+ LeafChild<End>
{
    fn to_range_path(self) -> PatternRangePath;
    fn complete(pattern: impl IntoPattern) -> Self;
    fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(query: P) -> Result<Self, (ErrorReason, Self)>;
    fn start_index<Trav: Traversable>(
        &self,
        trav: Trav,
    ) -> Child {
        self.role_leaf_child(&trav)
    }
}

pub trait RangePath: RootedPath {
    fn new_range(
        root: Self::Root,
        entry: usize,
        exit: usize,
    ) -> Self;
}

//impl PatternStart for PatternRangePath {}
//impl PatternEnd for PatternRangePath {}
//impl TraversalPath for PatternRangePath {
//    fn prev_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: Trav) -> Option<usize> {
//        if self.end.is_empty() {
//            D::pattern_index_prev(self.query.borrow(), self.exit)
//        } else {
//            let location = *self.end.last().unwrap();
//            let pattern = trav.graph().expect_pattern_at(&location);
//            D::pattern_index_prev(pattern, location.sub_index)
//        }
//    }
//}
