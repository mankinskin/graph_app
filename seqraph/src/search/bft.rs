use std::collections::VecDeque;
use std::iter::{Extend, FusedIterator};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use itertools::Itertools;

use crate::{ChildPath, Child, ChildLocation, Tokenize, Hypergraph, Vertexed, MatchDirection, Indexed};

#[derive(Clone)]
pub struct Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
    queue: VecDeque<(usize, T)>,
    iter_children: F,
}

impl<T, F, I> Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
    #[inline]
    pub fn new(root: T, iter_children: F) -> Self {
        Self {
            queue: VecDeque::from(vec![(0, root)]),
            iter_children,
        }
    }
}

impl<T, F, I> Iterator for Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
    type Item = (usize, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((depth, node)) = self.queue.pop_front() {
            let children = (self.iter_children)(&node);
            self.queue.extend(children.map(|child| (depth + 1, child)));

            Some((depth, node))
        } else {
            None
        }
    }
}

impl<T, F, I> FusedIterator for Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
}

pub(crate) trait BftNode {
    fn parent_node(start_path: ChildPath) -> Self;
    fn child_node(start_path: ChildPath, root: Child, end_path: ChildPath, next_child: Child) -> Self;
}
pub(crate) trait Traversable<T: Tokenize> {
    type Node: BftNode;
    type State;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>>;
    fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>>;
    fn parent_nodes(
        &self,
        root: Child,
        start_path: ChildPath,
    ) -> Vec<Self::Node> {
        let graph = &*self.graph();
        let vertex = root.vertex(&graph);
        let mut parents = vertex.get_parents().into_iter().collect_vec();
        // try parents in ascending width (might not be needed in indexing)
        parents.sort_unstable_by_key(|a| a.1.width);
        parents.into_iter()
            .map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent.pattern_indices
                    .iter()
                    .map(|&pi| {
                        //subgraph.add_index_parent(root.index, p, pi);
                        let mut start_path = start_path.clone();
                        start_path.push(ChildLocation::new(p, pi.pattern_id, pi.sub_index));
                        Self::Node::parent_node(start_path)
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
}
pub(crate) trait DirectedTraversalPolicy<'g, T: Tokenize, D: MatchDirection>: BreadthFirstTraversal<'g, T> {
    fn successor_nodes(
        trav: <Self as BreadthFirstTraversal<'g, T>>::Trav,
        mut start_path: ChildPath,
        query_next: Child,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        // find parent partition with matching context
        let loc@ChildLocation {
            parent: root,
            pattern_id,
            sub_index,
        } = start_path.pop().unwrap();
        {
            let graph = trav.graph();
            let parent_vertex = graph.expect_vertex_data(root.index());
            let child_patterns = parent_vertex.get_children();
            let pattern = child_patterns.get(&pattern_id).unwrap();
            if let Some(next_child) = D::next_child(pattern, sub_index) {
                // equal indices would have been found in find
                return if next_child.width > query_next.width {
                    Some(<Self::Trav as Traversable<T>>::Node::child_node(
                        start_path,
                        root,
                        vec![ChildLocation::new(root, pattern_id, D::index_next(sub_index).unwrap())],
                        next_child,
                    ))
                } else {
                    None
                }
                .into_iter()
                .collect_vec();
            }
        }
        <Self as BreadthFirstTraversal<'g, T>>::end_op(trav, start_path, root, loc)
    }
}
pub(crate) trait BreadthFirstTraversal<'g, T: Tokenize>: Sized {
    type Trav: Traversable<T>;
    fn end_op(
        trav: Self::Trav,
        start_path: ChildPath,
        root: Child,
        loc: ChildLocation,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node>;
}