pub mod cache;
pub mod child;
pub mod has_graph;
pub mod node;
pub mod pattern;
pub mod traceable;

use has_graph::HasGraph;

use crate::{
    graph::vertex::pattern::pattern_width,
    path::{
        RoleChildPath,
        accessors::{
            child::root::GraphRootChild,
            has_path::HasRolePath,
            role::End,
        },
        mutators::move_path::key::{
            AdvanceKey,
            TokenPosition,
        },
    },
    trace::cache::{
        TraceCache,
        key::{
            directed::{
                DirectedKey,
                down::DownKey,
                up::UpKey,
            },
            prev::ToPrev,
        },
        new::{
            NewChild,
            NewEntry,
            NewKind,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum TraceDirection {
    BottomUp,
    TopDown,
}

#[derive(Debug)]
pub struct TraceContext<G: HasGraph> {
    pub trav: G,
    pub cache: TraceCache,
}
impl<G: HasGraph> TraceContext<G> {
    pub fn trace_path<
        P: RoleChildPath + GraphRootChild<End> + HasRolePath<End>,
    >(
        &mut self,
        root_entry: usize,
        path: &P,
        root_up_pos: TokenPosition,
        add_edges: bool,
    ) {
        let graph = self.trav.graph();
        let root_exit = path.role_root_child_location::<End>();

        if add_edges
            && path.raw_child_path::<End>().is_empty()
            && graph.expect_is_at_end(&root_exit.clone())
        {
            return;
        }
        let root_parent = path.root_parent();
        let root_up_key = UpKey::new(root_parent.clone(), root_up_pos.into());
        let pattern = graph.expect_pattern_at(root_exit.clone());

        // path width
        let root_down_pos = root_up_pos
            + pattern
                .get(root_entry + 1..root_exit.sub_index)
                .map(pattern_width)
                .unwrap_or_default();

        let root_down_key =
            DownKey::new(root_parent.clone(), root_down_pos.into());
        let exit_down_key = DownKey::new(
            *graph.expect_child_at(root_exit.clone()),
            root_down_pos.into(),
        );
        let mut prev_key: DirectedKey = root_down_key.into();
        let mut target_key: DirectedKey = exit_down_key.into();
        self.cache.add_state(
            NewEntry {
                prev: prev_key.to_prev(0),
                kind: NewKind::Child(NewChild {
                    root: root_up_key.clone(),
                    target: target_key.clone(),
                    end_leaf: Some(root_exit.clone()),
                }),
            },
            add_edges,
        );
        for loc in path.raw_child_path::<End>() {
            prev_key = target_key.clone();
            let delta = graph.expect_child_offset(loc);
            prev_key.advance_key(delta);
            target_key = DirectedKey::down(
                *graph.expect_child_at(loc),
                *prev_key.pos.pos(),
            );
            self.cache.add_state(
                NewEntry {
                    prev: prev_key.to_prev(0),
                    kind: NewKind::Child(NewChild {
                        root: root_up_key.clone(),
                        target: target_key.clone(),
                        end_leaf: Some(loc.clone()),
                    }),
                },
                add_edges,
            );
        }
    }
}
