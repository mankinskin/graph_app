pub mod bands;

pub(crate) use bands::*;

use crate::{MatchDirection, Tokenize};

use super::*;

pub(crate) trait TraversalIterator<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<'a, 'g, T> + 'a,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
>: Iterator<Item = (usize, TraversalNode<Q>)> + Sized
{

    fn new(trav: &'a Trav, root: TraversalNode<Q>) -> Self;
    fn trav(&self) -> &'a Trav;
    fn cache_mut(&mut self) -> &mut TraversalCache<Q>;
    fn extend_nodes(&mut self, next_nodes: impl DoubleEndedIterator<Item=(usize, TraversalNode<Q>)>);
    fn cached_extend(&mut self, last_depth: usize, last_node: TraversalNode<Q>) {
        last_node.get_parent_path()
            .and_then(|path|
                self.cache_mut().bu_node(&last_node, path.entry())
                //None as Option<()>
            )
            .or_else(|| {
                // not a parent node or first time seeing parent
                let next_nodes = 
                    self.iter_children(&last_node)
                        .into_iter()
                        .map(|child| (last_depth + 1, child));
                self.extend_nodes(next_nodes);
                None
            });
    }
    fn iter_children(&mut self, node: &TraversalNode<Q>) -> Vec<TraversalNode<Q>> {
        match node.clone().into() {
            TraversalNode::Query(query) =>
                self.query_start(
                    query,
                ).unwrap_or_default(),
            TraversalNode::Parent(path, query) =>
                self.after_parent_nodes(
                    path,
                    query,
                ),
            TraversalNode::ToMatch(paths) =>
                self.to_match(
                    paths,
                ),
            TraversalNode::Match(path, query) =>
                self.after_match(
                    PathPair::GraphMajor(path, query),
                ),
            TraversalNode::MatchEnd(match_end, query) =>
                self.at_index_end(
                    &query,
                    &match_end
                ),
            _ => vec![],
        }
    }
    /// nodes generated when an index ended
    /// (parent nodes)
    fn at_index_end(
        &mut self,
        query: &FolderQuery<'a, 'g, T, D, Q, S>,
        match_end: &MatchEnd,
    ) -> Vec<TraversalNode<Q>> {
        self.cache_mut().bu_finished(match_end.root().index);
        S::next_parents(
            self.trav(),
            query,
            match_end,
        )
    }
    fn after_match(
        &mut self,
        paths: FolderPathPair<'a, 'g, T, D, Q, S>,
    ) -> Vec<TraversalNode<Q>> {
        let mode = paths.mode();
        let (mut path, query) = paths.unpack();
        assert!(!query.is_finished(self.trav()));
        if path.advance::<_, D, _>(self.trav()).is_some() && !path.is_finished(self.trav()) {
            vec![TraversalNode::to_match_node(PathPair::from_mode(path, query, mode))]
        } else {
            // possibly perform indexing
            let match_end = S::after_end_match(
                self.trav(),
                path,
            );
            // get next parents
            let parents = self.at_index_end(
                &query,
                &match_end
            );
            if parents.is_empty() {
                vec![
                    TraversalNode::match_end_node(
                        match_end.reduce_start::<_, D, _>(self.trav()),
                        query,
                    )
                ]
            } else {
                parents
            }
        }
    }
    /// nodes generated from a parent node
    /// (child successor or super-parent nodes)
    fn after_parent_nodes(
        &mut self,
        start: StartPath,
        query: FolderQuery<'a, 'g, T, D, Q, S>,
    ) -> Vec<TraversalNode<Q>> {
        assert!(!query.is_finished(self.trav()));
        match SearchPath::new_advanced::<_, D, _>(self.trav(), start) {
            Ok(path) =>
                vec![
                    TraversalNode::to_match_node(
                        PathPair::GraphMajor(path, query)
                    )
                ],
            Err(path) =>
                self.at_index_end(
                    &query,
                    &MatchEnd::from(path)
                ),
        }
    }
    /// match query position with graph position
    fn to_match(
        &mut self,
        paths: FolderPathPair<'a, 'g, T, D, Q, S>,
    ) -> Vec<TraversalNode<Q>> {
        let (mut path, mut query) = paths.unpack();
        if path.is_finished(self.trav()) || query.is_finished(self.trav()) {
            println!("uh oh");
        }
        let path_next = path.get_end::<_, D, _>(self.trav()).expect("Path at end");
        let query_next = query.get_end::<_, D, _>(self.trav()).expect("Query at end");
        match path_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                self.prefix_nodes(
                    path_next,
                    PathPair::GraphMajor(path, query),
                ),
            Ordering::Less =>
                self.prefix_nodes(
                    query_next,
                    PathPair::QueryMajor(query, path),
                ),
            Ordering::Equal =>
                if path_next == query_next {
                    path.add_match_width::<_, D, _>(self.trav());
                    vec![
                        if query.advance::<_, D, _>(self.trav()).is_some() && !query.is_finished(self.trav()) {
                            TraversalNode::match_node(
                                path,
                                query,
                            )
                        } else {
                            TraversalNode::query_end_node(TraversalResult::new(
                                path.reduce_end::<_, D, _>(self.trav()),
                                query,
                            ))
                        }
                    ]
                } else if path_next.width == 1 {
                    let prev_root = path.root().index;
                    let continued = self.cache_mut().bu_mismatch(prev_root);
                    let path = path.reduce_mismatch::<_, D, _>(self.trav());
                    (path.width() > 1).then(||
                        TraversalNode::mismatch_node(TraversalResult::new(
                            path,
                            query,
                        ))
                    )
                    .into_iter()
                    .chain(continued)
                    .collect()
                } else {
                    self.prefix_nodes(
                        path_next,
                        PathPair::GraphMajor(path.clone(), query.clone()),
                    )
                    .into_iter()
                    .chain(
                        self.prefix_nodes(
                            query_next,
                            PathPair::QueryMajor(query, path),
                        )
                    )
                    .collect_vec()
                }
        }
    }
    /// generate child nodes for index prefixes
    fn prefix_nodes(
        &self,
        index: Child,
        paths: FolderPathPair<'a, 'g, T, D, Q, S>,
    ) -> Vec<TraversalNode<Q>> {
        self.trav().graph()
            .expect_vertex_data(index)
            .get_child_patterns().iter()
            .sorted_unstable_by_key(|(_, p)| p.first().unwrap().width)
            .map(|(&pid, child_pattern)| {
                let sub_index = D::head_index(child_pattern.borrow());
                let mut paths = paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                TraversalNode::to_match_node(paths)
            })
            .collect_vec()
    }
    /// nodes generated from a query start node
    /// (query end or start parent nodes)
    fn query_start(
        &self,
        mut query: FolderQuery<'a, 'g, T, D, Q, S>,
    ) -> Option<Vec<TraversalNode<Q>>> {
        let start_index = query.get_end::<_, D, _>(self.trav())?;
        query.advance::<_, D, _>(self.trav())
            .and_then(|_|
                (!query.is_finished(self.trav())).then(||
                    S::gen_parent_nodes(
                        self.trav(),
                        &query,
                        start_index,
                        |p|
                            StartPath::Leaf(StartLeaf {
                                entry: p,
                                child: start_index,
                                width: start_index.width,
                            })
                    )
                )
            )
    }
}