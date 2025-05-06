use std::collections::VecDeque;

use crate::traversal::{
    compare::RootCursor,
    iterator::policy::DirectedTraversalPolicy,
    state::{
        child::{
            batch::{
                ChildIterator,
                ChildMatchState::{
                    self,
                    Mismatch,
                },
                ChildQueue,
            },
            ChildState,
        },
        parent::ParentState,
    },
    TraversalKind,
};
use context_trace::path::mutators::adapters::IntoAdvanced;

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

#[derive(Debug)]
pub struct RootSearchIterator<'a, K: TraversalKind> {
    pub ctx: &'a mut MatchContext,
    pub trav: &'a K::Trav,
}
impl<'a, K: TraversalKind> RootSearchIterator<'a, K> {
    pub fn new(
        trav: &'a K::Trav,
        ctx: &'a mut MatchContext,
    ) -> Self {
        //assert!(!batch.is_empty());

        Self {
            //nodes: batch.parents.into_iter().map(Parent).collect(),
            ctx,
            trav,
        }
    }

    pub fn find_root_cursor(mut self) -> Option<RootCursor<&'a K::Trav>> {
        self.find_map(|root| root).map(|child| RootCursor {
            trav: self.trav,
            state: child,
        })
    }
}

impl<'a, K: TraversalKind> Iterator for RootSearchIterator<'a, K> {
    type Item = Option<ChildState>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ctx.nodes.pop_front().and_then(|node| {
            PolicyNode::<'_, K>::new(node, &self.trav).consume()
        }) {
            Some(Append(next)) => {
                self.ctx.nodes.extend(next);
                Some(None)
            },
            Some(Match(cs)) => {
                self.ctx.nodes.clear();
                Some(Some(cs))
            },
            Some(Pass) => Some(None),
            None => None,
        }
    }
}
#[derive(Debug, new)]
struct PolicyNode<'a, K: TraversalKind>(TraceNode, &'a K::Trav);

#[derive(Debug)]
pub enum TraceStep {
    Append(Vec<TraceNode>),
    Match(ChildState),
    Pass,
}
use TraceStep::*;

impl<'a, K: TraversalKind> PolicyNode<'a, K> {
    fn consume(self) -> Option<TraceStep> {
        match self.0 {
            Parent(parent) => match parent.into_advanced(&self.1) {
                Ok(state) => PolicyNode::<K>::new(
                    Child(ChildQueue::from(state.child)),
                    self.1,
                )
                .consume(),
                Err(parent) => Some(Append(
                    K::Policy::next_batch(&self.1, &parent)
                        .into_iter()
                        .flat_map(|batch| batch.parents)
                        .map(Parent)
                        .collect(),
                )),
            },
            Child(queue) => {
                let mut child_iter =
                    ChildIterator::<&K::Trav>::new(&self.1, queue);
                match child_iter.next() {
                    Some(Some(ChildMatchState::Match(cs))) => Some(Match(cs)),
                    Some(Some(Mismatch(_))) => Some(Pass),
                    Some(None) =>
                        Some(Append(vec![Child(child_iter.children)])),
                    None => None,
                }
            },
        }
    }
}
