pub mod bands;

pub(crate) use bands::*;

use crate::*;

use super::*;

pub(crate) trait TraversalIterator<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<'a, 'g, T> + 'a,
    Q: TraversalQuery,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, R, Trav=Trav>,
    R: ResultKind = BaseResult,
>: Iterator<Item = (usize, TraversalNode<R, Q>)> + Sized
{

    fn new(trav: &'a Trav, root: TraversalNode<R, Q>) -> Self;
    fn trav(&self) -> &'a Trav;
    fn cache_mut(&mut self) -> &mut TraversalCache<R, Q>;
    fn extend_nodes(&mut self, next_nodes: impl IntoIterator<IntoIter=impl DoubleEndedIterator<Item=(usize, TraversalNode<R, Q>)>>);
    fn cached_extend(
        &mut self,
        last_depth: usize,
        last_node: TraversalNode<R, Q>,
    ) {
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
                        .map(|child| (last_depth + 1, child))
                        .collect_vec();
                self.extend_nodes(next_nodes);
                None
            });
    }
    fn iter_children(
        &mut self,
        node: &TraversalNode<R, Q>
    ) -> Vec<TraversalNode<R, Q>> {
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
        query: &FolderQuery<'a, 'g, T, D, Q, R, S>,
        postfix: &R::Postfix,
    ) -> Vec<TraversalNode<R, Q>> {
        let root = postfix.root_child();
        println!("at index end {:?}", root);
        self.cache_mut().bu_finished(root.index);
        S::next_parents(
            self.trav(),
            query,
            postfix,
        )
    }
    /// runs after each index/query match
    fn after_match(
        &mut self,
        paths: FolderPathPair<'a, 'g, T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
        let mode = paths.mode();
        let (mut path, query) = paths.unpack();
        assert!(!query.is_finished(self.trav()));
        if path.advance::<_, D, _>(self.trav()).is_some() && !path.is_finished(self.trav()) {
            vec![TraversalNode::to_match_node(PathPair::from_mode(path, query, mode))]
        } else {
            // at end of index
            // possibly perform indexing
            let match_end = S::after_end_match(
                self.trav(),
                path.into(),
            );
            // get next parents
            let parents = self.at_index_end(
                &query,
                &match_end.clone()
            );
            if parents.is_empty() {
                println!("no more parents {:#?}", match_end);
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
    }
    /// nodes generated from a parent node
    /// (child successor or super-parent nodes)
    fn after_parent_nodes(
        &mut self,
        start: R::Primer,
        query: FolderQuery<'a, 'g, T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
        assert!(!query.is_finished(self.trav()));
        match R::Advanced::new_advanced::<_, D, _, _>(self.trav(), start) {
            Ok(path) =>
                vec![
                    TraversalNode::to_match_node(
                        PathPair::GraphMajor(path, query)
                    )
                ],
            Err(path) =>
                self.at_index_end(
                    &query,
                    &R::Postfix::from(path)
                ),
        }
    }
    /// match query position with graph position
    fn to_match(
        &mut self,
        paths: FolderPathPair<'a, 'g, T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
        let (mut path, mut query) = paths.unpack();
        if path.is_finished(self.trav()) || query.is_finished(self.trav()) {
            println!("uh oh");
        }
        let path_next = path.get_end::<_, D, _>(self.trav()).expect("Path at end");
        let query_next = query.get_end::<_, D, _>(self.trav()).expect("Query at end");

        println!("matching query {:?}", query_next);
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
                                //R::Found::from_advanced::<_, D, _>(
                                //    path,
                                //    self.trav()
                                //).into_result(query)
                            )
                        }
                    ]
                } else if path_next.width() == 1 && query_next.width() == 1 {
                    // mismatch
                    let root = path.root_child().index;
                    let continued = self.cache_mut().bu_mismatch(root);
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
        paths: FolderPathPair<'a, 'g, T, D, Q, R, S>,
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
        mut query: FolderQuery<'a, 'g, T, D, Q, R, S>,
    ) -> Option<Vec<TraversalNode<R, Q>>> {
        let start_index = query.get_end::<_, D, _>(self.trav())?;
        query.advance::<_, D, _>(self.trav())
            .and_then(|_|
                (!query.is_finished(self.trav())).then(||
                    S::gen_parent_nodes(
                        self.trav(),
                        &query,
                        start_index,
                        |p|
                            R::Primer::from(StartLeaf {
                                entry: p,
                                child: start_index,
                                width: start_index.width,
                            })
                    )
                )
            )
    }
}