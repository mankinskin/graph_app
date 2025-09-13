use std::{collections::VecDeque};

use itertools::Itertools;

use crate::graph::{utils::cover::frequency::FrequencyCover, vocabulary::{
    entry::VertexCtx,
    NGramId,
    Vocabulary,
}};
use context_trace::graph::vertex::{
    child::Child,
    key::VertexKey,
    wide::Wide,
    VertexIndex,
};

use crate::graph::{
    labelling::frequency::FrequencyCtx,
    traversal::pass::TraversalPass
};
use derive_more::{
    Deref,
    DerefMut,
};

pub trait Queue<P: TraversalPass>: FromIterator<P::NextNode> {
    fn extend_layer(&mut self, iter: impl IntoIterator<Item = <P as TraversalPass>::NextNode>);
    fn finish_layer(&mut self);
    fn pop_front(&mut self) -> Option<P::NextNode>;
    fn is_empty(&self) -> bool;
}
type NodeQueue<P> = VecDeque<<P as TraversalPass>::NextNode>;
#[derive(Debug, Deref, Default)]
pub struct LayeredQueue<P: TraversalPass> {
    #[deref]
    queue: NodeQueue<P>,
    layer: NodeQueue<P>,
}
impl<P: TraversalPass> FromIterator<P::NextNode> for LayeredQueue<P> {
    fn from_iter<T: IntoIterator<Item = P::NextNode>>(iter: T) -> Self {
        Self {
            queue: FromIterator::from_iter(iter),
            layer: Default::default(),
        }
    }
}
impl<P: TraversalPass> Queue<P> for LayeredQueue<P> {
    fn extend_layer(&mut self, iter: impl IntoIterator<Item = <P as TraversalPass>::NextNode>) {
        self.layer.extend(iter)
    }
    fn finish_layer(&mut self) {
        self.queue.extend(
            self.layer.drain(..),
        )
    }
    fn pop_front(&mut self) -> Option<P::NextNode> {
        self.queue.pop_front()
    }
    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[derive(Debug, Deref, Default)]
pub struct LinearQueue<P: TraversalPass> {
    queue: NodeQueue<P>,
}
impl<P: TraversalPass> FromIterator<P::NextNode> for LinearQueue<P> {
    fn from_iter<T: IntoIterator<Item = P::NextNode>>(iter: T) -> Self {
        Self {
            queue: FromIterator::from_iter(iter)
        }
    }
}
impl<P: TraversalPass> Queue<P> for LinearQueue<P> {
    fn extend_layer(&mut self, iter: impl IntoIterator<Item = <P as TraversalPass>::NextNode>) {
        self.queue.extend(iter)
    }
    fn finish_layer(&mut self) {
    }
    fn pop_front(&mut self) -> Option<P::NextNode> {
        self.queue.pop_front()
    }
    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[derive(Debug, Deref, Default)]
pub struct SortedQueue {
    pub queue: VecDeque<NGramId>,
}
impl FromIterator<NGramId> for SortedQueue {
    fn from_iter<T: IntoIterator<Item = NGramId>>(iter: T) -> Self {
        let mut v = Self {
            queue: VecDeque::default(),
        };
        v.extend_layer(iter);
        v
    }
}

impl Queue<FrequencyCtx<'_>> for SortedQueue
{
    fn extend_layer(
        &mut self,
        iter: impl IntoIterator<Item = NGramId>,
    )
    {
        self.queue.extend(iter);
        self.queue = self
            .queue
            .drain(..)
            .sorted_by_key(|i| std::cmp::Reverse(i.width()))
            .dedup()
            .collect();
    }
    fn finish_layer(&mut self) {
    }
    fn pop_front(&mut self) -> Option<NGramId> {
        self.queue.pop_front()
    }
    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
