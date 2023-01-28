use crate::*;

pub trait NodePath<R>: RootChild<R> + Send + Clone + Eq + Debug {}
impl<R, T: RootChild<R> + Send + Clone + Eq + Debug> NodePath<R> for T {}


pub trait DirectedTraversalPolicy<
    T: Tokenize,
    D: MatchDirection,
    R: ResultKind,
>: Sized + Send + Sync + Unpin {

    type Trav: Traversable<T> + TraversalFolder<T, D, Self, R>;

    /// Executed after last child of index matched
    fn at_postfix(
        _trav: &Self::Trav,
        path: R::Primer,
    ) -> R::Postfix;
    /// nodes generated when an index ended
    /// (parent nodes)
    fn next_parents(
        trav: &Self::Trav,
        query: &R::Query,
        postfix: &R::Postfix,
    ) -> Vec<ParentState<R>> {
        Self::gen_parent_states(
            trav,
            query,
            postfix.root_parent(),
            |p, trav| postfix.clone().into_primer(p)
        )
    }
    /// generates parent nodes
    fn gen_parent_states<
        B: (Fn(ChildLocation, &Self::Trav) -> R::Primer) + Copy,
    >(
        trav: &Self::Trav,
        query: &R::Query,
        index: Child,
        build_start: B,
    ) -> Vec<ParentState<R>> {
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
                ParentState {
                    path: build_start(p, trav),
                    query: query.clone(),
                }
            })
            .collect()
        
    }
}