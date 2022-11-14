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

#[async_trait]
pub(crate) trait DirectedTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    R: ResultKind + 'a,
>: Sized + Send + Sync + Unpin {

    type Trav: Traversable<'a, 'g, T> + 'a;
    //type Primer: PathPrimer + From<R::Result<StartPath>> + GraphEntry;
    type Folder: TraversalFolder<'a, 'g, T, D, Q, R, Trav=Self::Trav,
    // Primer=StartPath
    >;

    /// Executed after last child of index matched
    async fn after_end_match(
        _trav: &'a Self::Trav,
        path: R::Primer,
    ) -> R::Postfix;
    /// nodes generated when an index ended
    /// (parent nodes)
    async fn next_parents(
        trav: &'a Self::Trav,
        query: &Q,
        primer: &R::Postfix,
    ) -> Vec<TraversalNode<R, Q>> {
        Self::gen_parent_nodes(
            trav,
            query,
            primer.root_child(),
            |p| primer.clone().append::<_, D, _>(trav, p)
        ).await
    }
    /// generates parent nodes
    async fn gen_parent_nodes<
        B: (Fn(ChildLocation) -> F) + Send + Sync + Copy,
        F: Future<Output=R::Primer> + Send,
    >(
        trav: &'a Self::Trav,
        query: &Q,
        index: Child,
        build_start: B,
    ) -> Vec<TraversalNode<R, Q>> {
        futures::stream::iter(trav.graph().await
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
            .map(|p| async move {
                TraversalNode::parent_node(
                    build_start(p).await,
                    query.clone(),
                )
            }.into_stream())
        )
        .flatten()
        .collect()
        .await
    }
}