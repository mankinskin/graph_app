use std::collections::VecDeque;

use crate::{
    compare::{
        iterator::CompareIterator,
        parent::ParentCompareState,
        state::{
            ChildMatchState,
            CompareState,
        },
    },
    r#match::root_cursor::RootCursor,
    traversal::{
        policy::DirectedTraversalPolicy,
        TraversalKind,
    },
};
use context_trace::{
    path::mutators::adapters::IntoAdvanced,
    trace::child::iterator::ChildQueue,
};

use derive_new::new;
pub mod end;
pub mod root_cursor;

#[derive(Debug, new)]
pub struct MatchContext {
    #[new(default)]
    pub nodes: VecDeque<TraceNode>,
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
        Self { ctx, trav }
    }

    pub fn find_root_cursor(mut self) -> Option<RootCursor<&'a K::Trav>> {
        self.find_map(|root| root).map(|state| RootCursor {
            trav: self.trav,
            state,
        })
    }
}

impl<'a, K: TraversalKind> Iterator for RootSearchIterator<'a, K> {
    type Item = Option<CompareState>;

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

#[derive(Debug)]
pub enum TraceStep {
    Append(Vec<TraceNode>),
    Match(CompareState),
    Pass,
}
use TraceStep::*;

#[derive(Debug)]
pub enum TraceNode {
    Parent(ParentCompareState),
    Child(ChildQueue<CompareState>),
}
use TraceNode::*;

#[derive(Debug, new)]
struct PolicyNode<'a, K: TraversalKind>(TraceNode, &'a K::Trav);

impl<'a, K: TraversalKind> PolicyNode<'a, K> {
    fn consume(self) -> Option<TraceStep> {
        match self.0 {
            Parent(parent) => match parent.into_advanced(&self.1) {
                Ok(state) => PolicyNode::<K>::new(
                    Child(ChildQueue::from_iter([state.child])),
                    self.1,
                )
                .consume(),
                Err(parent) => Some(Append(
                    K::Policy::next_batch(&self.1, &parent)
                        .into_iter()
                        .flat_map(|batch| batch.parents)
                        .map(|parent_state| ParentCompareState {
                            parent_state,
                            cursor: parent.cursor.clone(),
                        })
                        .map(Parent)
                        .collect(),
                )),
            },
            Child(queue) => {
                let mut compare_iter =
                    CompareIterator::<&K::Trav>::new(&self.1, queue);
                match compare_iter.next() {
                    Some(Some(ChildMatchState::Match(cs))) => Some(Match(cs)),
                    Some(Some(ChildMatchState::Mismatch(_))) => Some(Pass),
                    Some(None) =>
                        Some(Append(vec![Child(compare_iter.children.queue)])),
                    None => None,
                }
            },
        }
    }
}
