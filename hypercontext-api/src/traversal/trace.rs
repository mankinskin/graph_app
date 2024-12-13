use std::fmt::Display;

use crate::{
    graph::vertex::pattern::pattern_width,
    traversal::{
        cache::{
            key::{
                prev::ToPrev, DirectedKey, DownKey, UpKey
            },
            state::end::{
                EndKind,
                EndState,
            },
            TraversalCache,
        },
        result::kind::RoleChildPath,
        traversable::{
            TravToken,
            Traversable,
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

impl TraversalCache {
    pub fn trace_path<
        Trav: Traversable,
        P: RoleChildPath + GraphRootChild<End> + HasRolePath<End>,
    >(
        &mut self,
        trav: &Trav,
        root_entry: usize,
        path: &P,
        root_up_pos: TokenPosition,
        add_edges: bool,
    ) where
        TravToken<Trav>: Display,
    {
        let graph = trav.graph();
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
        self.add_state(
            trav,
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
            self.add_state(
                trav,
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
pub trait Trace {
    fn trace<Trav: Traversable>(
        &self,
        trav: &Trav,
        cache: &mut TraversalCache,
    );
}

impl Trace for EndState {
    fn trace<Trav: Traversable>(
        &self,
        trav: &Trav,
        cache: &mut TraversalCache,
    ) {
        match &self.kind {
            EndKind::Range(p) => {
                let root_entry = p.path.role_root_child_location::<Start>().sub_index;
                cache.trace_path(trav, root_entry, &p.path, self.root_pos, true)
            }
            EndKind::Prefix(p) => cache.trace_path(trav, 0, &p.path, self.root_pos, true),
            _ => {}
        }
    }
}
//impl Trace for ChildState {
//    fn trace<Trav: Traversable>(&self, trav: &Trav, cache: &mut TraversalCache) {
//        cache.trace_path(
//            trav,
//            self.paths.path.role_root_child_location::<Start>().sub_index,
//            &self.paths.path,
//            self.root_pos,
//            false,
//        );
//    }
//}