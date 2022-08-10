use super::*;
use std::cmp::Ordering;
use crate::{
    *,
    Child,
    ChildLocation,
    Tokenize,
    MatchDirection,
    TraversalOrder,
};

pub(crate) trait DirectedTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery
>: Sized {

    type Trav: Traversable<'a, 'g, T>;
    type Folder: TraversalFolder<'a, 'g, T, D, Q, Trav=Self::Trav>;

    /// generates start node
    fn after_match_end(
        _trav: &'a Self::Trav,
        path: SearchPath,
    ) -> MatchEnd {
        StartPath::from(path).into()
    }
    /// nodes generated from a query start node
    /// (query end or start parent nodes)
    fn query_start(
        trav: &'a Self::Trav,
        mut query: FolderQuery<'a, 'g, T, D, Q, Self>,
    ) -> Option<Vec<FolderNode<'a, 'g, T, D, Q, Self>>> {
        let start_index = query.get_end::<_, D, _>(trav)?;
        query.advance::<_, D, _>(trav)
            .and_then(|_|
                (!query.is_finished(trav)).then(||
                    Self::gen_parent_nodes(
                        trav,
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
    /// generates parent nodes
    fn gen_parent_nodes(
        trav: &'a Self::Trav,
        query: &FolderQuery<'a, 'g, T, D, Q, Self>,
        index: Child,
        build_start: impl Fn(ChildLocation) -> StartPath,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        trav.graph()
            .expect_vertex_data(index)
            .get_parents()
            .iter()
            .flat_map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent.pattern_indices
                    .iter()
                    .cloned()
                    .map(move |pi| {
                        ChildLocation::new(p, pi.pattern_id, pi.sub_index)
                    })
            })
            .sorted_unstable_by(|a, b| TraversalOrder::cmp(a, b))
            .map(|p|
                ToTraversalNode::parent_node(
                    build_start(p),
                    query.clone(),
                )
            )
            .collect_vec()
    }
    /// nodes generated when an index ended
    /// (parent nodes)
    fn at_index_end(
        trav: &'a Self::Trav,
        query: &FolderQuery<'a, 'g, T, D, Q, Self>,
        match_end: &MatchEnd,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        Self::gen_parent_nodes(
            trav,
            query,
            match_end.root(),
            |p| match_end.clone().append::<_, D, _>(trav, p)
        )
    }
    /// nodes generated from a parent node
    /// (child successor or super-parent nodes)
    fn after_parent_nodes(
        trav: &'a Self::Trav,
        start: StartPath,
        query: FolderQuery<'a, 'g, T, D, Q, Self>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        assert!(!query.is_finished(trav));
        match SearchPath::new_advanced::<_, D, _>(trav, start) {
            Ok(path) =>
                vec![
                    ToTraversalNode::to_match_node(
                        PathPair::GraphMajor(path, query)
                    )
                ],
            Err(path) =>
                Self::at_index_end(
                    trav,
                    &query,
                    &MatchEnd::from(path)
                ),
        }
    }
    fn after_match(
        trav: &'a Self::Trav,
        paths: FolderPathPair<'a, 'g, T, D, Q, Self>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        let mode = paths.mode();
        let (mut path, query) = paths.unpack();
        assert!(!query.is_finished(trav));
        if path.advance::<_, D, _>(trav).is_some() && !path.is_finished(trav) {
            vec![ToTraversalNode::to_match_node(PathPair::from_mode(path, query, mode))]
        } else {
            let match_end = Self::after_match_end(trav, path);
            let parents = Self::at_index_end(
                trav,
                &query,
                &match_end
            );
            if parents.is_empty() {
                vec![
                    ToTraversalNode::match_end_node(
                        match_end,
                        query,
                    )
                ]
            } else {
                parents
            }
        }
    }
    fn to_match(
        trav: &'a Self::Trav,
        paths: FolderPathPair<'a, 'g, T, D, Q, Self>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        let (mut path, mut query) = paths.unpack();
        if path.is_finished(trav) || query.is_finished(trav) {
            println!("uh oh");
        }
        let path_next = path.get_end::<_, D, _>(trav).expect("Path at end");
        let query_next = query.get_end::<_, D, _>(trav).expect("Query at end");
        match path_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                Self::prefix_nodes(
                    trav,
                    path_next,
                    PathPair::GraphMajor(path, query),
                ),
            Ordering::Less =>
                Self::prefix_nodes(
                    trav,
                    query_next,
                    PathPair::QueryMajor(query, path),
                ),
            Ordering::Equal =>
                if path_next == query_next {
                    path.add_match_width::<_, D, _>(trav);
                    vec![
                        if query.advance::<_, D, _>(trav).is_some() && !query.is_finished(trav) {
                            ToTraversalNode::match_node(
                                path,
                                query,
                            )
                        } else {
                            ToTraversalNode::query_end_node(Some(TraversalResult::new(
                                path.reduce_end::<_, D, _>(trav),
                                query,
                            )))
                        }
                    ]
                } else if path_next.width == 1 {
                    let path = path.reduce_mismatch::<_, D, _>(trav);
                    if path.get_exit_pos() == path.get_entry_pos() {
                        vec![]
                    } else {
                        vec![
                            ToTraversalNode::mismatch_node(PathPair::GraphMajor(path, query))
                        ]
                    }
                } else {
                    Self::prefix_nodes(
                        trav,
                        path_next,
                        PathPair::GraphMajor(path.clone(), query.clone()),
                    )
                    .into_iter()
                    .chain(
                        Self::prefix_nodes(
                            trav,
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
        trav: &'a Self::Trav,
        index: Child,
        paths: FolderPathPair<'a, 'g, T, D, Q, Self>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        trav.graph()
            .expect_vertex_data(index)
            .get_child_patterns().iter()
            .sorted_unstable_by_key(|(_, p)| p.first().unwrap().width)
            .map(|(&pid, child_pattern)| {
                let sub_index = D::head_index(child_pattern.borrow());
                let mut paths = paths.clone();
                paths.push_major(ChildLocation::new(index, pid, sub_index));
                ToTraversalNode::to_match_node(paths)
            })
            .collect_vec()
    }
}