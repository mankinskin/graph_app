use crate::*;

pub trait NodePath<R>: RootChild<R> + Send + Clone + Eq + Debug {}
impl<R, T: RootChild<R> + Send + Clone + Eq + Debug> NodePath<R> for T {}


pub trait DirectedTraversalPolicy: Sized {

    type Trav: Traversable + TraversalFolder<Self>;

    /// Executed after last child of index matched
    fn at_postfix(
        _trav: &Self::Trav,
        path: Primer,
    ) -> Postfix;
    /// nodes generated when an index ended
    /// (parent nodes)
    fn next_parents(
        trav: &Self::Trav,
        postfix: &Postfix,
        query: &QueryState,
    ) -> Vec<ParentState> {
        Self::gen_parent_states(
            trav,
            query,
            postfix.root_parent(),
            |trav, p| postfix.clone().into_primer(trav, p)
        )
    }
    /// generates parent nodes
    fn gen_parent_states<
        B: (Fn(&Self::Trav, ChildLocation) -> Primer) + Copy,
    >(
        trav: &Self::Trav,
        query: &QueryState,
        index: Child,
        build_start: B,
    ) -> Vec<ParentState> {
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
                    path: build_start(trav, p),
                    query: query.clone(),
                }
            })
            .collect()
        
    }
}