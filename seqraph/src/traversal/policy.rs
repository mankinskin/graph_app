use super::*;
use crate::{
    *,
    Child,
    ChildLocation,
    Tokenize,
    MatchDirection,
    TraversalOrder,
};

pub(crate) trait NodePath: RootChild + PathAppend<Result = StartPath> + Send + Clone + Eq {}
impl<T: RootChild + PathAppend<Result = StartPath> + Send + Clone + Eq> NodePath for T {}

pub(crate) trait PostfixPath: NodePath + PathReduce + IntoMatchEndStartPath {}
impl<T: NodePath + PathReduce + IntoMatchEndStartPath> PostfixPath for T {}

pub(crate) trait DirectedTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
>: Sized {

    type Trav: Traversable<'a, 'g, T> + 'a;
    type AfterEndMatch: PostfixPath;
    type Folder: TraversalFolder<'a, 'g, T, D, Q, R, Trav=Self::Trav, AfterEndMatch=Self::AfterEndMatch>;

    /// Executed after last child of index matched
    fn after_end_match(
        _trav: &'a Self::Trav,
        path: StartPath,
    ) -> Self::AfterEndMatch;
    /// nodes generated when an index ended
    /// (parent nodes)
    fn next_parents(
        trav: &'a Self::Trav,
        query: &Q,
        match_end: &MatchEnd<StartPath>,
    ) -> Vec<TraversalNode<Self::AfterEndMatch, Q>> {
        Self::gen_parent_nodes(
            trav,
            query,
            match_end.root_child(),
            |p| match_end.clone().append::<_, D, _>(trav, p)
        )
    }
    /// generates parent nodes
    fn gen_parent_nodes(
        trav: &'a Self::Trav,
        query: &Q,
        index: Child,
        build_start: impl Fn(ChildLocation) -> StartPath,
    ) -> Vec<TraversalNode<Self::AfterEndMatch, Q>> {
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
                TraversalNode::parent_node(
                    build_start(p),
                    query.clone(),
                )
            )
            .collect_vec()
    }
}