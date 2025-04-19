use itertools::Itertools;
use std::fmt::Debug;

use crate::traversal::{
    container::order::TraversalOrder,
    state::parent::ParentState,
    ParentBatch,
};
use context_trace::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::child::ChildLocation,
        },
    },
    path::{
        accessors::root::GraphRoot,
        mutators::raise::PathRaise,
    },
    trace::has_graph::HasGraph,
};

pub trait DirectedTraversalPolicy: Sized + Debug {
    type Trav: HasGraph;

    /// nodes generated when an index ended
    /// (parent nodes)
    fn next_batch(
        trav: &Self::Trav,
        parent: &ParentState,
    ) -> Option<ParentBatch> {
        let batch = Self::gen_parent_batch(
            trav,
            parent.path.root_parent(),
            |trav, p| {
                let mut parent = parent.clone();
                parent.path_raise(trav, p);
                parent
            },
        );
        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    }
    /// generates parent nodes
    fn gen_parent_batch<
        B: (Fn(&Self::Trav, ChildLocation) -> ParentState) + Copy,
    >(
        trav: &Self::Trav,
        index: Child,
        build_parent: B,
    ) -> ParentBatch {
        ParentBatch {
            parents: trav
                .graph()
                .expect_vertex(index)
                .get_parents()
                .iter()
                .flat_map(|(i, parent)| {
                    let p = Child::new(i, parent.width);
                    parent.pattern_indices.iter().cloned().map(move |pi| {
                        ChildLocation::new(p, pi.pattern_id, pi.sub_index)
                    })
                })
                .sorted_by(|a, b| TraversalOrder::cmp(b, a))
                .map(|p| build_parent(trav, p))
                .collect(),
        }
    }
}
