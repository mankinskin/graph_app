use key::*;
use state::*;
use std::fmt::Display;

use crate::{
    HashMap,
    traversal::{
        cache::{
            entry::{
                new::{
                    NewChild,
                    NewEntry,
                    NewKind,
                },
                position::PositionCache,
                vertex::VertexCache,
            },
            key::{
                prev::ToPrev,
                target::TargetKey,
            },
            labelled_key::vkey::{
                labelled_key,
                VertexCacheKey,
            },
            state::{
                query::QueryState,
                start::StartState,
            },
        },
        context::{
            QueryContext,
            TraversalContext,
        },
        folder::TraversalFolder,
        iterator::traverser::ExtendStates,
        path::{
            accessors::{
                child::root::GraphRootChild,
                has_path::HasRolePath,
                role::End,
            },
            mutators::move_path::key::{
                AdvanceKey,
                TokenLocation,
            },
        },
        result_kind::RoleChildPath,
        traversable::{
            Traversable,
            TravToken,
        },
    },
};
use crate::graph::vertex::{
    child::Child,
    has_vertex_index::HasVertexIndex,
    pattern::pattern_width,
};

pub mod entry;
pub mod key;
pub mod labelled_key;
pub mod state;
pub mod trace;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalCache {
    pub(crate) entries: HashMap<VertexCacheKey, VertexCache>,
}

impl TraversalCache {
    pub fn new<'a, Folder: TraversalFolder>(
        folder: &'a Folder,
        start_index: Child,
        query_root: &QueryContext,
        query_state: QueryState,
    ) -> (Folder::Iterator<'a>, Self)
    where
        TravToken<Folder>: Display,
    {
        let mut entries = HashMap::default();
        entries.insert(
            labelled_key(folder, start_index),
            VertexCache::start(start_index),
        );
        let mut start = StartState {
            index: start_index,
            key: UpKey::new(
                start_index,
                0.into(), //TokenLocation(start_index.width()).into(),
            ),
            query: query_state,
        };

        let mut cache = Self { entries };
        let mut states = Folder::Iterator::from(folder);

        let init = {
            let mut ctx = TraversalContext::new(&query_root, &mut cache, &mut states);
            start
                .next_states(&mut ctx)
                .into_states()
                .into_iter()
                .map(|n| (1, n))
        };
        states.extend(init);
        (states, cache)
    }
    pub fn add_state<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        state: NewEntry,
        add_edges: bool,
    ) -> (DirectedKey, bool)
    where
        TravToken<Trav>: Display,
    {
        let key = state.target_key();
        if let Some(ve) = self.entries.get_mut(&key.index.vertex_index()) {
            if let Some(_) = ve.get_mut(&key.pos) {
                (key, false)
            } else {
                //drop(ve);

                let pe = PositionCache::new(self, trav, key, state, add_edges);
                let ve = self.entries.get_mut(&key.index.vertex_index()).unwrap();
                ve.insert(&key.pos, pe);
                (key, true)
            }
        } else {
            self.new_vertex(trav, key, state, add_edges);
            (key, true)
        }
    }
    pub fn trace_path<
        Trav: Traversable,
        P: RoleChildPath + GraphRootChild<End> + HasRolePath<End>,
    >(
        &mut self,
        trav: &Trav,
        root_entry: usize,
        path: &P,
        root_up_pos: TokenLocation,
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
                .map(|p| pattern_width(p))
                .unwrap_or_default();

        let root_down_key = DownKey::new(path.root_parent(), root_down_pos.into());
        let exit_down_key = DownKey::new(graph.expect_child_at(&root_exit), root_down_pos.into());
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
                        root: root_up_key.into(),
                        target: target_key,
                        end_leaf: Some(*loc),
                    }),
                },
                add_edges,
            );
        }
    }
    fn new_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        key: DirectedKey,
        state: NewEntry,
        add_edges: bool,
    ) where
        TravToken<Trav>: Display,
    {
        let mut ve = VertexCache::from(key.index);
        let pe = PositionCache::new(self, trav, key, state, add_edges);
        ve.insert(&key.pos, pe);
        self.entries.insert(labelled_key(trav, key.index), ve);
    }
    pub fn continue_waiting(
        &mut self,
        key: &UpKey,
    ) -> Vec<(usize, TraversalState)> {
        self.get_mut(&(DirectedKey::from(*key)))
            .unwrap()
            .waiting
            .drain(..)
            .map(|(d, s)| (d, s.into()))
            .collect()
    }
    pub fn force_mut<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        key: &DirectedKey,
    ) -> &mut PositionCache
    where
        TravToken<Trav>: Display,
    {
        if !self.exists(&key) {
            let pe = PositionCache::start(key.index);
            if let Some(ve) = self.get_vertex_mut(&key.index) {
                ve.insert(&key.pos, pe);
            } else {
                let mut ve = VertexCache::from(key.index);
                ve.insert(&key.pos, pe);
                self.entries.insert(labelled_key(trav, key.index), ve);
            }
        }
        self.expect_mut(key)
    }
    pub fn get_vertex(
        &self,
        key: &Child,
    ) -> Option<&VertexCache> {
        self.entries.get(&key.index.vertex_index())
    }
    pub fn get_vertex_mut(
        &mut self,
        key: &Child,
    ) -> Option<&mut VertexCache> {
        self.entries.get_mut(&key.index.vertex_index())
    }
    pub fn expect_vertex(
        &self,
        key: &Child,
    ) -> &VertexCache {
        self.get_vertex(key).unwrap()
    }
    pub fn expect_vertex_mut(
        &mut self,
        key: &Child,
    ) -> &mut VertexCache {
        self.get_vertex_mut(key).unwrap()
    }
    pub fn get(
        &self,
        key: &DirectedKey,
    ) -> Option<&PositionCache> {
        self.get_vertex(&key.index).and_then(|ve| ve.get(&key.pos))
    }
    pub fn get_mut(
        &mut self,
        key: &DirectedKey,
    ) -> Option<&mut PositionCache> {
        self.get_vertex_mut(&key.index).and_then(|ve| {
            //println!("get_entry positions {:#?}: {:#?}", key, ve.positions);
            ve.get_mut(&key.pos)
        })
    }
    pub fn expect(
        &self,
        key: &DirectedKey,
    ) -> &PositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(
        &mut self,
        key: &DirectedKey,
    ) -> &mut PositionCache {
        self.get_mut(key).unwrap()
    }
    pub fn exists_vertex(
        &self,
        key: &Child,
    ) -> bool {
        if let Some(_) = self.entries.get(&key.vertex_index()) {
            true
        } else {
            false
        }
    }
    pub fn exists(
        &self,
        key: &DirectedKey,
    ) -> bool {
        if let Some(ve) = self.entries.get(&key.index.vertex_index()) {
            ve.get(&key.pos).is_some()
        } else {
            false
        }
    }
}
