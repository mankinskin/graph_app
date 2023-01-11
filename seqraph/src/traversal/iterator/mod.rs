pub mod bands;

pub use bands::*;

use crate::*;

use super::*;


pub trait TraversalIterator<
    'a, 
    T: Tokenize,
    D: MatchDirection,
    Trav: Traversable<T> + 'a,
    Q: BaseQuery,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Trav>,
    R: ResultKind = BaseResult,
>: Iterator<Item = (usize, TraversalNode<R, Q>)> + Sized
{

    fn new(trav: &'a Trav, root: Q) -> Option<Self>;
    fn trav(&self) -> &'a Trav;
    fn cache_mut(&mut self) -> &mut TraversalCache<R, Q>;
    fn next_nodes(
        &mut self,
        node: TraversalNode<R, Q>,
    ) -> (CacheKey, Vec<TraversalNode<R, Q>>) {
        let cache = self.cache_mut();
        match cache.add_node(&node) {
            Ok(key) => (key, match node {
                // compute next nodes
                TraversalNode::Start(node) =>
                    self.query_start(
                        key,
                        node.query,
                    ).unwrap_or_default(),
                TraversalNode::Parent(_prev, node) =>
                    self.on_parent_node(key, node),
                TraversalNode::Child(_prev, node) =>
                    self.on_child(key, node),
                TraversalNode::MatchEnd(_prev, _entry, match_end, query) =>
                    S::next_parents(
                        self.trav(),
                        key,
                        &query,
                        &match_end,
                    ),
                //TraversalNode::Match(path, query) =>
                //    self.after_match(
                //        PathPair::GraphMajor(path, query),
                //    ),
                _ => vec![],
            }),
            Err(key) => (key, vec![])
        }
    }
    /// nodes generated when an index ended
    /// (parent nodes)
    //fn at_index_end(
    //    &mut self,
    //    path: R::Primer,
    //    query: FolderQuery<T, D, Q, R, S>,
    //) -> Vec<TraversalNode<R, Q>> {
    //    self.after_index(
    //        path.into(),
    //        query
    //    )
    //}
    fn after_index(
        &mut self,
        key: CacheKey,
        primer: R::Primer,
        query: FolderQuery<T, D, Q, R, S>,
    ) -> Vec<TraversalNode<R, Q>> {
        // get next parents
        let entry = primer.root_child_location();
        let postfix = primer.into();
        let parents = S::next_parents(
            self.trav(),
            key,
            &query,
            &postfix,
        );
        if parents.is_empty() {
            //println!("no more parents {:#?}", match_end);
            vec![
                TraversalNode::MatchEnd(
                    key,
                    entry,
                    postfix.into_simplified::<_, D, _>(self.trav()),
                    query,
                )
            ]
        } else {
            parents
        }
    }
    //fn at_index_finished(
    //    &mut self,
    //    query: FolderQuery<T, D, Q, R, S>,
    //    nodes: BinaryHeap<WaitingNode<R, Q>>,
    //) -> Vec<TraversalNode<R, Q>> {
    //    //println!("at index end {:?}", root);
    //    // possibly perform indexing
    //    let match_end = S::at_postfix(
    //        self.trav(),
    //        nodes,
    //    );
    //    self.after_index(
    //        query,
    //    )
    //}
    /// runs after each index/query match
    //fn after_match(
    //    &mut self,
    //    paths: FolderPathPair<T, D, Q, R, S>,
    //) -> Vec<TraversalNode<R, Q>> {
    //    let mode = paths.mode();
    //    let (mut path, query) = paths.unpack();
    //    assert!(!query.is_finished(self.trav()));
    //    if path.advance::<_, D, _>(self.trav()).is_some() && !path.is_finished(self.trav()) {
    //        vec![
    //            TraversalNode::child_node(PathPair::from_mode(path, query, mode))
    //        ]
    //    } else {
    //        // at end of index
    //        self.at_index_end(
    //            query,
    //            path.into(),
    //        )
    //    }
    //}
    /// nodes generated from a parent node
    /// (child successor or super-parent nodes)
    fn on_parent_node(
        &mut self,
        key: CacheKey,
        node: ParentNode<R, Q>,
    ) -> Vec<TraversalNode<R, Q>> {
        // todo: solve the "is finished" predicate with a type (how to relate to specific trav state?)
        assert!(!node.query.is_finished(self.trav()));
        // create path to next child
        match R::Primer::into_advanced::<_, D, _>(node.path, self.trav()) {
            Ok(path) =>
                vec![TraversalNode::Child(key, ChildNode {
                    root: key,
                    paths: PathPair::GraphMajor(path, node.query)
                })],
            Err(path) =>
                // no next child
                self.after_index(
                    key,
                    path,
                    node.query,
                ),
        }
    }
    /// match query position with graph position
    fn on_child(
        &mut self,
        key: CacheKey,
        node: ChildNode<R, Q>,
    ) -> Vec<TraversalNode<R, Q>> {
        let ChildNode {
            root,
            paths,
        } = node;
        let mode = paths.mode();

        let (mut path, mut query) = paths.unpack();
        if path.is_finished(self.trav()) || query.is_finished(self.trav()) {
            println!("can't match finished paths");
        }

        let path_next = path.role_path_child::<End, _, _>(self.trav());
        let query_next = query.path_child(self.trav());

        // compare next child
        //println!("matching query {:?}", query_next);
        match path_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                self.prefix_nodes(
                    key,
                    root, 
                    path_next,
                    PathPair::GraphMajor(path, query),
                ),
            Ordering::Less =>
                self.prefix_nodes(
                    key,
                    root, 
                    query_next,
                    PathPair::QueryMajor(query, path),
                ),
            Ordering::Equal =>
                if path_next == query_next {
                    // match
                    path.add_match_width::<_, D, _>(self.trav());
                    vec![
                        if query.advance::<_, D, _>(self.trav()).is_some() && !query.is_finished(self.trav()) {
                            TraversalNode::Child(key, ChildNode {
                                root,
                                paths: PathPair::from_mode(path, query, mode)
                            })
                        } else {
                            path.child_path_mut::<End>().simplify::<_, D, _>(self.trav());
                            let (entry, path) = if path.raw_child_path::<End>().is_empty() && <_ as RootChildPos<Start>>::root_child_pos(&path) == <_ as RootChildPos<End>>::root_child_pos(&path) {
                                (None, R::Found::from(path.pop_path::<_, D, _>(self.trav())))
                            } else {
                                (path.role_path_child_location::<End>(), R::Found::from_advanced::<_, D, _>(path, self.trav()))
                            };
                            TraversalNode::QueryEnd(
                                key,
                                entry,
                                path.into_result(query)
                            )
                        }
                    ]
                } else if path_next.width() == 1 && query_next.width() == 1 {
                    // mismatch
                    todo!("handle waiting nodes");
                    let continued = vec![];
                    //self.cache_mut().on_bu_mismatch(path.cache_key()).unwrap_or_default();
                    path.child_path_mut::<End>().retract::<_, D, _, R>(self.trav());

                    let (entry, found) = if path.raw_child_path::<End>().is_empty()
                        && path.child_pos::<Start>() == path.child_pos::<End>()
                    {
                        (None, R::Found::from(path.pop_path::<_, D, _>(self.trav())))
                    } else {
                        (path.role_path_child_location::<End>(), R::Found::from_advanced::<_, D, _>(path, self.trav()))
                    };

                    (found.width() > 1).then(||
                        TraversalNode::Mismatch(
                            key,
                            entry,
                            found.into_result(query),
                        )
                    )
                    .into_iter()
                    .chain(continued)
                    .collect()
                } else {
                    // expand nodes
                    self.prefix_nodes(
                        key,
                        root, 
                        path_next,
                        PathPair::GraphMajor(path.clone(), query.clone()),
                    )
                    .into_iter()
                    .chain(
                        self.prefix_nodes(
                            key,
                            root,
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
        key: CacheKey,
        root: CacheKey,
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
                TraversalNode::Child(key, ChildNode {
                    root,
                    paths,
                })
            })
            .collect_vec()
    }
    /// nodes generated from a query start node
    /// (query end or start parent nodes)
    fn query_start(
        &self,
        key: CacheKey,
        mut query: FolderQuery<T, D, Q, R, S>,
    ) -> Option<Vec<TraversalNode<R, Q>>> {
        let start_index = query.path_child(self.trav());
        if let Some(_) = query.advance::<_, D, _>(self.trav()) {
            if !query.is_finished(self.trav()) {
                Some(S::gen_parent_nodes(
                    self.trav(),
                    key,
                    &query,
                    start_index,
                    |p, trav| {
                        R::Primer::from(ChildPath {
                            path: vec![p],
                            child: start_index,
                            width: start_index.width,
                            token_pos: trav.graph().expect_pattern_range_width(&p, 0..p.sub_index),
                            _ty: Default::default(),
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