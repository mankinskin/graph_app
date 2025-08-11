use context_insert::interval::{
    init::InitInterval,
    IntervalGraph,
};
use context_trace::{
    graph::{
        self,
        vertex::{
            child::Child,
            location::child::ChildLocation,
        },
    },
    path::{
        accessors::child::root::RootChild,
        structs::rooted::role_path::IndexEndPath,
    },
    trace::{
        cache::TraceCache,
        has_graph::{
            HasGraph,
            HasGraphMut,
        },
    },
};
use derive_new::new;

use crate::expansion::ExpansionLink;

use super::context::ReadCtx;

#[derive(Debug, new)]
pub struct ComplementBuilder {
    link: ExpansionLink,
}

impl ComplementBuilder {
    pub fn build(
        self,
        trav: &mut ReadCtx,
    ) -> Child {
        // Get the root index from the postfix path
        let root = self.link.root_postfix.root_child(trav);

        // Calculate the complement end bound from the postfix path
        let intersection_start = self.link.root_postfix.root_entry;

        if intersection_start == 0 {
            // If intersection is at the beginning, no complement exists
            return root;
        }

        // Build the trace cache for the complement path
        let complement_cache = self.build_complement_trace_cache(trav, root);

        // Create InitInterval for the complement range
        let init_interval = InitInterval {
            root,
            cache: complement_cache,
            end_bound: intersection_start,
        };

        // Use IntervalGraph to efficiently create the complement
        let interval_graph =
            IntervalGraph::from((trav.graph_mut(), init_interval));

        // The IntervalGraph should provide the complement child
        interval_graph.result_child() // or whatever method extracts the Child
    }

    fn build_complement_trace_cache(
        &self,
        trav: &ReadCtx,
        root: Child,
    ) -> TraceCache {
        use context_trace::{
            path::mutators::move_path::retract::Retract,
            trace::{
                command::PrefixCommand,
                traceable::Traceable,
                TraceCtx,
            },
        };

        // Initialize cache with the root
        let cache = TraceCache::new(root);

        // Get the intersection point from the postfix path
        let intersection_start = self.link.root_postfix.root_entry;

        // If intersection is at the beginning, no complement exists
        if intersection_start == 0 {
            return cache;
        }

        // Create a mutable copy of the postfix path to retract to the previous index
        let mut complement_postfix = self.link.root_postfix.clone();

        // Use retract API to move to the index before the intersection
        // This ensures we point to the last index of the complement range
        std::iter::repeat_with(|| complement_postfix.retract(trav))
            .take_while(|result| result.is_continue())
            .take(1) // Only retract once to get to the previous index
            .count();

        // Convert the retracted postfix path to a prefix path for the PrefixCommand
        // The postfix path goes from root to complement end, so we convert it to
        // a prefix path that represents the same range but as a top-down traversal
        let complement_prefix_path: IndexEndPath = complement_postfix.into();

        // Use PrefixCommand to build the trace cache efficiently
        let prefix_command = PrefixCommand {
            path: complement_prefix_path,
            add_edges: true,
        };

        // Create trace context and execute the command
        let mut trace_ctx = TraceCtx { trav, cache };

        prefix_command.trace(&mut trace_ctx);
        trace_ctx.cache
    }
}
