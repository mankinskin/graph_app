use std::fmt::Display;

use derive_more::derive::IntoIterator;
use key::directed::DirectedKey;
use label_key::vkey::{
    VertexCacheKey,
    labelled_key,
};

use crate::{
    HashMap,
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    trace::cache::{
        entry::{
            new::NewEntry,
            position::PositionCache,
            vertex::VertexCache,
        },
        key::props::TargetKey,
    },
};

use super::traversable::{
    TravToken,
    Traversable,
};

pub mod entry;
pub mod key;
pub mod label_key;

#[derive(Clone, Debug, PartialEq, Eq, Default, IntoIterator)]
pub struct TraceCache {
    pub entries: HashMap<VertexCacheKey, VertexCache>,
}

impl Extend<(VertexCacheKey, VertexCache)> for TraceCache {
    fn extend<T: IntoIterator<Item = (VertexCacheKey, VertexCache)>>(
        &mut self,
        iter: T,
    ) {
        for (k, v) in iter {
            if let Some(c) = self.entries.get_mut(&k) {
                assert!(c.index == v.index);
                c.bottom_up.extend(v.bottom_up);
                c.top_down.extend(v.top_down);
            } else {
                self.entries.insert(k, v);
            }
        }
    }
}
impl TraceCache {
    pub fn new<Trav: Traversable>(
        trav: &Trav,
        start_index: Child,
    ) -> Self
    where
        TravToken<Trav>: Display,
    {
        let mut entries = HashMap::default();
        entries.insert(
            labelled_key(trav, start_index),
            VertexCache::start(start_index),
        );
        Self { entries }
    }
    pub fn add_state<Trav: Traversable, E: Into<NewEntry>>(
        &mut self,
        trav: &Trav,
        state: E,
        add_edges: bool,
    ) -> (DirectedKey, bool)
    where
        TravToken<Trav>: Display,
    {
        let state = state.into();
        let key = state.target_key();
        if let Some(ve) = self.entries.get_mut(&key.index.vertex_index()) {
            if ve.get_mut(&key.pos).is_some() {
                (key, false)
            } else {
                //drop(ve);

                let pe = PositionCache::new(self, trav, key, state, add_edges);
                let ve =
                    self.entries.get_mut(&key.index.vertex_index()).unwrap();
                ve.insert(&key.pos, pe);
                (key, true)
            }
        } else {
            self.new_vertex(trav, key, state, add_edges);
            (key, true)
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
    pub fn force_mut<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        key: &DirectedKey,
    ) -> &mut PositionCache
    where
        TravToken<Trav>: Display,
    {
        if !self.exists(key) {
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
        self.entries.contains_key(&key.vertex_index())
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
