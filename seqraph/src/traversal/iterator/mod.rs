pub mod bands;

use std::collections::BinaryHeap;

pub use bands::*;

use crate::*;

use super::*;


pub trait TraversalIterator<
    'a, 
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T> + 'a,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
    R: ResultKind = BaseResult,
>: Iterator<Item = (usize, TraversalNode<R, Q>)> + Sized
{

    fn new(trav: &'a Trav, root: TraversalNode<R, Q>) -> Self;
    fn trav(&self) -> &'a Trav;
    fn cache_mut(&mut self) -> &mut TraversalCache<R, Q>;
    fn next_nodes(
        &mut self,
        last_depth: usize,
        last_node: TraversalNode<R, Q>,
    ) -> Vec<(usize, TraversalNode<R, Q>)> {
        match last_node.into() {
            TraversalNode::Query(query) =>
                self.query_start(
                    query,
                ).unwrap_or_default(),
            TraversalNode::Parent(node) =>
                self.on_parent_node(node),
            TraversalNode::ToMatch(paths) =>
                self.to_match(
                    paths,
                ),
            TraversalNode::Match(path, query) =>
                self.after_match(
                    PathPair::GraphMajor(path, query),
                ),
            TraversalNode::MatchEnd(match_end, query) =>
                S::next_parents(
                    self.trav(),
                    &query,
                    &match_end
                ),
            _ => vec![],
        }
        .into_iter()
        .map(|child| (last_depth + 1, child))
        .collect_vec()
    }
    /// nodes generated when an index ended
    /// (parent nodes)
    fn at_index_end(
        &mut self,
        query: FolderQuery<T, D, Q, R, S>,
        path: R::Primer,
    ) -> Vec<TraversalNode<R, Q>> {
        match self.cache_mut().bu_index_end(path.cache_key()) {
            OnIndexEnd::Finished(nodes) => {
                self.at_index_finished(
                    query,
                    nodes,
                )
            },
            OnIndexEnd::Waiting => vec![],
        }
    }
    fn at_index_finished(
        &mut self,
        query: FolderQuery<T, D, Q, R, S>,
        nodes: BinaryHeap<WaitingNode<R, Q>>,
    ) -> Vec<TraversalNode<R, Q>> {
        //println!("at index end {:?}", root);
        // possibly perform indexing
        let match_end = S::at_postfix(
            self.trav(),
            nodes,
        );
        // get next parents
        let parents = S::next_parents(
            self.trav(),
            &query,
            &match_end,
        );
        if parents.is_empty() {
            //println!("no more parents {:#?}", match_end);
            vec![
                TraversalNode::match_end_node(
                    match_end.into_reduced::<_, D, _>(self.trav()),
                    query,
                )
            ]
        } else {
            parents
        }
    }
    /// runs after each index/query match
    fn after_match(
        &mut self,
        paths: FolderPathPair<T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
        let mode = paths.mode();
        let (mut path, query) = paths.unpack();
        assert!(!query.is_finished(self.trav()));
        if path.advance::<_, D, _>(self.trav()).is_some() && !path.is_finished(self.trav()) {
            vec![TraversalNode::to_match_node(PathPair::from_mode(path, query, mode))]
        } else {
            // at end of index
            self.at_index_end(
                query,
                path.into(),
            )
        }
    }
    /// nodes generated from a parent node
    /// (child successor or super-parent nodes)
    fn on_parent_node(
        &mut self,
        node: ParentNode<R, Q>,
    ) -> Vec<TraversalNode<R, Q>> {
        assert!(!node.query.is_finished(self.trav()));

        match self.cache_mut().on_parent_node(node) {
            OnParent::First => {
                match R::Advanced::new_advanced::<_, D, _, _>(self.trav(), node.path) {
                    Ok(path) =>
                        vec![
                            TraversalNode::to_match_node(
                                PathPair::GraphMajor(path, node.query)
                            )
                        ],
                    Err(path) =>
                        self.at_index_end(
                            node.query,
                            path
                        ),
                }
            },
            OnParent::Last(nodes) => nodes.map(|nodes|
                self.at_index_finished(
                    node.query,
                    nodes,
                )
            ).unwrap_or_default(),
            OnParent::Waiting => vec![]
        }
    }
    /// match query position with graph position
    fn to_match(
        &mut self,
        paths: FolderPathPair<T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
        let (mut path, mut query) = paths.unpack();
        if path.is_finished(self.trav()) || query.is_finished(self.trav()) {
            println!("uh oh");
        }
        let path_next = path.get_end::<_, D, _>(self.trav()).expect("Path at end");
        let query_next = query.get_end::<_, D, _>(self.trav()).expect("Query at end");

        //println!("matching query {:?}", query_next);
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
                    // match
                    path.add_match_width::<_, D, _>(self.trav());
                    vec![
                        if query.advance::<_, D, _>(self.trav()).is_some() && !query.is_finished(self.trav()) {
                            TraversalNode::match_node(
                                path,
                                query,
                            )
                        } else {
                            path.end_match_path_mut().reduce::<_, D, _>(self.trav());
                            TraversalNode::query_end_node(
                                if path.end_path().is_empty() && path.get_entry_pos() == path.get_exit_pos() {
                                    R::Found::from(path.pop_path::<_, D, _>(self.trav()))
                                } else {
                                    R::Found::from_advanced::<_, D, _>(path, self.trav())
                                }.into_result(query)
                            )
                        }
                    ]
                } else if path_next.width() == 1 && query_next.width() == 1 {
                    // mismatch
                    let continued = self.cache_mut().on_bu_mismatch(path.cache_key());
                    path.end_match_path_mut().retract::<_, D, _, R>(self.trav());

                    let found = if path.end_path().is_empty() && path.get_entry_pos() == path.get_exit_pos() {
                        R::Found::from(path.pop_path::<_, D, _>(self.trav()))
                    } else {
                        R::Found::from_advanced::<_, D, _>(path, self.trav())
                    };

                    (found.width() > 1).then(||
                        TraversalNode::mismatch_node(
                            found.into_result(query),
                        )
                    )
                    .into_iter()
                    .chain(continued)
                    .collect()
                } else {
                    // expand nodes
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
        paths: FolderPathPair<T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
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
        mut query: FolderQuery<T, D, Q, R, S>,
    ) -> Option<Vec<TraversalNode<R, Q>>> {
        let start_index = query.get_end::<_, D, _>(self.trav())?;
        if let Some(_) = query.advance::<_, D, _>(self.trav()) {
            if !query.is_finished(self.trav()) {
                Some(S::gen_parent_nodes(
                    self.trav(),
                    &query,
                    start_index,
                    |p, trav| {
                        R::Primer::from(StartLeaf {
                            entry: p,
                            child: start_index,
                            width: start_index.width,
                            token_pos: trav.graph().expect_pattern_range_width(&p, 0..p.sub_index),
                        })
                    }
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}