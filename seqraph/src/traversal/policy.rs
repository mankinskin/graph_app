use super::*;
use crate::{
    *,
    Child,
    ChildLocation,
    Tokenize,
    MatchDirection,
    TraversalOrder,
};

pub(crate) trait NodePath: RootChild + Send + Clone + Eq + Debug {}
impl<T: RootChild + Send + Clone + Eq + Debug> NodePath for T {}


pub(crate) trait DirectedTraversalPolicy<
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
>: Sized + Send + Sync + Unpin {

    type Trav: Traversable<T>;
    //type Primer: PathPrimer + From<R::Result<StartPath>> + GraphEntry;
    type Folder: TraversalFolder<T, D, Q, R//, Trav=Self::Trav,
    // Primer=StartPath
    >;

    /// Executed after last child of index matched
    fn at_postfix(
        _trav: &Self::Trav,
        path: R::Primer,
    ) -> R::Postfix;
    /// nodes generated when an index ended
    /// (parent nodes)
    fn next_parents(
        trav: &Self::Trav,
        query: &Q,
        primer: &R::Postfix,
    ) -> Vec<TraversalNode<R, Q>> {
        Self::gen_parent_nodes(
            trav,
            query,
            primer.root_child(),
            |p, trav| primer.clone().append::<_, D, _>(trav, p)
        )
    }
    /// generates parent nodes
    fn gen_parent_nodes<
        B: (Fn(ChildLocation, &Self::Trav) -> R::Primer) + Copy,
    >(
        trav: &Self::Trav,
        query: &Q,
        index: Child,
        build_start: B,
    ) -> Vec<TraversalNode<R, Q>> {
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
            .map(|p| {
                TraversalNode::parent_node(
                    build_start(p, trav),
                    query.clone(),
                )
            })
            .collect()
        
    }
}