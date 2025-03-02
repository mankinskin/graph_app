use itertools::Itertools;
use std::fmt::Debug;

use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::child::ChildLocation,
        },
    },
    path::{
        accessors::{
            child::root::RootChild,
            role::PathRole,
            root::GraphRoot,
        },
        mutators::raise::PathRaise,
    },
    traversal::{
        container::order::TraversalOrder,
        state::bottom_up::parent::ParentState,
        traversable::Traversable,
    },
};

pub trait NodePath<R: PathRole>: RootChild<R> + Send + Clone + Eq + Debug {}

impl<R: PathRole, T: RootChild<R> + Send + Clone + Eq + Debug> NodePath<R> for T {}

pub trait DirectedTraversalPolicy: Sized + Debug {
    type Trav: Traversable;

    /// nodes generated when an index ended
    /// (parent nodes)
    fn next_parents(
        trav: &Self::Trav,
        parent: &ParentState,
    ) -> Vec<ParentState> {
        Self::gen_parent_states(trav, parent.path.root_parent(), |trav, p| {
            let mut parent = parent.clone();
            parent.path_raise(trav, p);
            parent
        })
    }
    /// generates parent nodes
    fn gen_parent_states<B: (Fn(&Self::Trav, ChildLocation) -> ParentState) + Copy>(
        trav: &Self::Trav,
        index: Child,
        build_parent: B,
    ) -> Vec<ParentState> {
        trav.graph()
            .expect_vertex(index)
            .get_parents()
            .iter()
            .flat_map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent
                    .pattern_indices
                    .iter()
                    .cloned()
                    .map(move |pi| ChildLocation::new(p, pi.pattern_id, pi.sub_index))
            })
            .sorted_by(|a, b| TraversalOrder::cmp(b, a))
            .map(|p| build_parent(trav, p))
            .collect()
    }
}
