use std::collections::VecDeque;

use crate::traversal::{
    compare::RootCursor,
    state::{
        child::batch::ChildQueue,
        parent::{
            batch::{
                ParentBatch,
                RootSearchIterator,
            },
            ParentState,
        },
    },
    OptGen::{
        self,
        Pass,
        Yield,
    },
    TraversalKind,
};
use derive_new::new;

#[derive(Debug, new)]
pub struct MatchContext {
    #[new(default)]
    pub nodes: VecDeque<TraceNode>,
}

#[derive(Debug)]
pub enum TraceNode {
    Parent(ParentState),
    Child(ChildQueue),
}
use TraceNode::*;

//#[derive(Debug, new)]
//pub struct MatchIterator<'a, K: TraversalKind>(
//    &'a K::Trav,
//    &'a mut MatchContext,
//);

//impl<'a, K: TraversalKind> Iterator for MatchIterator<'a, K> {
//    type Item = OptGen<RootCursor<&'a K::Trav>>;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        let MatchIterator(trav, ctx) = self;
//        if let Some(batch) = ctx.batches.pop_front() {
//            // one parent level (batched)
//            Some(
//                match RootSearchIterator::<K>::new(trav, )
//                    // find parent with a match
//                    .find_root_cursor()
//                {
//                    // root found
//                    Some(root_cursor) => {
//                        // drop other candidates
//                        //ctx.batches.clear();
//                        // TODO: add cache for path to parent
//                        Yield(root_cursor)
//                    },
//                    // continue with
//                    None => {
//                        //ctx.batches.extend(next.into_iter().sorted().flat_map(
//                        //    |parent| K::Policy::next_batch(&trav, &parent),
//                        //));
//                        Pass
//                    },
//                },
//            )
//        } else {
//            // no more parents
//            None
//        }
//    }
//}
