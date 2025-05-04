use crate::traversal::{
    compare::RootCursor,
    iterator::{
        policy::DirectedTraversalPolicy,
        r#match::{
            MatchContext,
            TraceNode,
        },
    },
    state::{
        child::{
            batch::{
                ChildIterator,
                ChildMatchState,
                ChildQueue,
            },
            ChildState,
        },
        parent::{
            batch::{
                ChildMatchState::Mismatch,
                TraceNode::{
                    Child,
                    Parent,
                },
            },
            ParentState,
        },
    },
    TraversalKind,
};
use context_trace::path::mutators::adapters::IntoAdvanced;
use derive_more::derive::{
    Deref,
    DerefMut,
};
use derive_new::new;
use std::collections::VecDeque;
#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct ParentBatch {
    pub parents: VecDeque<ParentState>,
}

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
        self.ctx
            .nodes
            .pop_front()
            .and_then(|node| {
                PolicyNode::<'_, K>::new(node, &self.trav).consume()
            })
            .map(|step| match step {
                Append(next) => {
                    self.ctx.nodes.extend(next);
                    None
                },
                Match(cs) => {
                    self.ctx.nodes.clear();
                    Some(cs)
                },
                Pass => None,
            })
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
                child_iter.next().map(|res| match res {
                    Some(ChildMatchState::Match(cs)) => Match(cs),
                    Some(Mismatch(_)) => Pass,
                    None => Append(vec![Child(child_iter.children)]),
                })
            },
        }
    }
}
