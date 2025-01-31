use std::fmt::Display;

use crate::{
    graph::vertex::pattern::pattern_width,
    traversal::{
        cache::{
            key::{
                prev::ToPrev, DirectedKey, DownKey, UpKey
            },
            TraversalCache,
        },
        result::kind::RoleChildPath,
        traversable::{
            TravToken,
            Traversable,
        },
        state::end::{
            EndKind,
            EndState,
        },
    },
    path::{
        accessors::{
            child::root::GraphRootChild,
            has_path::HasRolePath,
            role::{
                End,
                Start,
            },
        },
        mutators::move_path::key::{
            AdvanceKey,
            TokenPosition,
        },
    },
};

use crate::traversal::cache::entry::new::{
    NewChild,
    NewEntry,
    NewKind,
};

#[derive(Debug)]
pub struct TraceContext<'a, Trav: Traversable> {
    pub trav: &'a Trav,
    pub cache: &'a mut TraversalCache
}
impl<Trav: Traversable> TraceContext<'_, Trav> {
    pub fn trace_path<
        P: RoleChildPath + GraphRootChild<End> + HasRolePath<End>,
    >(
        &mut self,
        root_entry: usize,
        path: &P,
        root_up_pos: TokenPosition,
        add_edges: bool,
    ) where
        TravToken<Trav>: Display,
    {
        let graph = self.trav.graph();
        let root_exit = path.role_root_child_location::<End>();

        if add_edges
            && path.raw_child_path::<End>().is_empty()
            && graph.expect_is_at_end(&root_exit)
        {
            return;
        }
        let root_up_key = UpKey::new(path.root_parent(), root_up_pos.into());
        let pattern = graph.expect_pattern_at(root_exit);

        // path width
        let root_down_pos = root_up_pos
            + pattern
                .get(root_entry + 1..root_exit.sub_index)
                .map(pattern_width)
                .unwrap_or_default();

        let root_down_key = DownKey::new(path.root_parent(), root_down_pos.into());
        let exit_down_key = DownKey::new(graph.expect_child_at(root_exit), root_down_pos.into());
        let mut prev_key: DirectedKey = root_down_key.into();
        let mut target_key = exit_down_key.into();
        self.cache.add_state(
            self.trav,
            NewEntry {
                prev: prev_key.to_prev(0),
                kind: NewKind::Child(NewChild {
                    root: root_up_key,
                    target: target_key,
                    end_leaf: Some(root_exit),
                }),
            },
            add_edges,
        );
        for loc in path.raw_child_path::<End>() {
            prev_key = target_key;
            let delta = graph.expect_child_offset(loc);
            prev_key.advance_key(delta);
            target_key = DirectedKey::down(graph.expect_child_at(loc), *prev_key.pos.pos());
            self.cache.add_state(
                self.trav,
                NewEntry {
                    //root_pos: root_up_pos,
                    prev: prev_key.to_prev(0),
                    kind: NewKind::Child(NewChild {
                        root: root_up_key,
                        target: target_key,
                        end_leaf: Some(*loc),
                    }),
                },
                add_edges,
            );
        }
    }
}
pub trait TraceInit {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<'_, Trav>,
    );
}

impl TraceInit for EndState {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<'_, Trav>,
    ) {
        match &self.kind {
            EndKind::Range(p) => {
                let root_entry = p.path.role_root_child_location::<Start>().sub_index;
                ctx.trace_path(root_entry, &p.path, self.root_pos, true)
            }
            EndKind::Prefix(p) => ctx.trace_path(0, &p.path, self.root_pos, true),
            _ => {}
        }
    }
}