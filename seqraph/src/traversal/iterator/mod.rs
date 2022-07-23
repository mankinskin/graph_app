pub mod bands;

pub(crate) use bands::*;

use crate::{MatchDirection, Tokenize};

use super::*;

pub(crate) trait TraversalIterator<
    'a: 'g,
    'g,
    T: Tokenize,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
>: Iterator<Item = (usize, FolderNode<'a, 'g, T, D, Q, S>)>
{
    fn new(trav: &'a Trav, root: FolderNode<'a, 'g, T, D, Q, S>) -> Self;
    fn iter_children(trav: &'a Trav, node: &FolderNode<'a, 'g, T, D, Q, S>) -> Vec<FolderNode<'a, 'g, T, D, Q, S>> {
        match node.clone().into() {
            TraversalNode::Query(query) =>
                S::query_start(
                    trav,
                    query,
                ),
            TraversalNode::Parent(path, query) =>
                S::after_parent_nodes(
                    trav,
                    path,
                    query,
                ),
            TraversalNode::ToMatch(paths) =>
                S::to_match(
                    trav,
                    paths,
                ),
            TraversalNode::Match(path, query, _prev_query) =>
                S::after_match(
                    trav,
                    PathPair::GraphMajor(path, query),
                ),
            TraversalNode::MatchEnd(match_end, query) =>
                S::at_index_end(
                    trav,
                    query,
                    match_end
                ),
            _ => vec![],
        }
    }
}