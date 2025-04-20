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
            role::{
                End,
                PathRole,
            },
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
pub struct TraceContext<'a, G: HasGraph> {
    pub trav: &'a G,
    pub cache: &'a mut TraceCache,
}
impl<'a, G: HasGraph> TraceContext<'a, G> {
    pub fn trace_rooted_end_path<
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
        //let root_up_key = UpKey::new(root_parent.clone(), root_up_pos.into());
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
        let prev_key: DirectedKey = root_down_key.into();
        let target_key: DirectedKey = exit_down_key.into();
        self.cache.add_state(
            NewEntry {
                prev: prev_key.clone().to_prev(0),
                kind: NewKind::Child(NewChild {
                    //root: root_up_key.clone(),
                    target: target_key.clone(),
                    end_leaf: Some(root_exit.clone()),
                }),
            },
            add_edges,
        );
        self.trace_path::<_, End>(path, target_key, add_edges)
    }
    pub fn trace_path<
        P: RoleChildPath + GraphRootChild<Role> + HasRolePath<Role>,
        Role: PathRole,
    >(
        &mut self,
        path: &P,
        mut target_key: DirectedKey,
        add_edges: bool,
    ) {
        let graph = self.trav.graph();
        for loc in path.raw_child_path::<Role>() {
            let mut prev = target_key.clone();
            let delta = graph.expect_child_offset(loc);
            prev.advance_key(delta);
            target_key =
                DirectedKey::down(*graph.expect_child_at(loc), *prev.pos.pos());
            self.cache.add_state(
                NewEntry {
                    prev: prev.to_prev(0),
                    kind: NewKind::Child(NewChild {
                        //root: root_up_key.clone(),
                        target: target_key.clone(),
                        end_leaf: Some(loc.clone()),
                    }),
                },
                add_edges,
            );
        }
    }
}
