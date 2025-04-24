use crate::{
    HashMap,
    graph::vertex::{
        VertexIndex,
        child::Child,
        has_vertex_index::HasVertexIndex,
    },
    trace::cache::{
        key::props::TargetKey,
        position::PositionCache,
        vertex::VertexCache,
    },
};
use derive_more::derive::IntoIterator;
use key::directed::DirectedKey;
use new::EditKind;

pub mod key;
pub mod new;
pub mod position;
pub mod vertex;

pub type StateDepth = usize;

#[derive(Clone, Debug, PartialEq, Eq, Default, IntoIterator)]
pub struct TraceCache {
    pub entries: HashMap<VertexIndex, VertexCache>,
}
impl TraceCache {
    pub fn new(start_index: Child) -> Self {
        let mut entries = HashMap::default();
        entries.insert(
            start_index.vertex_index(),
            VertexCache::start(start_index),
        );
        Self { entries }
    }
    pub fn add_state<E: Into<EditKind>>(
        &mut self,
        edit: E,
        add_edges: bool,
    ) -> (DirectedKey, bool) {
        let edit = edit.into();
        let key = edit.target_key();
        if let Some(ve) = self.entries.get_mut(&key.index.vertex_index()) {
            if ve.get_mut(&key.pos).is_some() {
                (key, false)
            } else {
                let pe = PositionCache::new(self, key.clone(), edit, add_edges);
                let ve =
                    self.entries.get_mut(&key.index.vertex_index()).unwrap();
                ve.insert(&key.pos, pe);
                (key, true)
            }
        } else {
            self.new_entry(key.clone(), edit, add_edges);
            (key, true)
        }
    }
    fn new_entry(
        &mut self,
        key: DirectedKey,
        edit: EditKind,
        add_edges: bool,
    ) {
        let mut ve = VertexCache::from(key.index.clone());
        let pe = PositionCache::new(self, key.clone(), edit, add_edges);
        ve.insert(&key.pos, pe);
        self.entries.insert(key.index.vertex_index(), ve);
    }
    pub fn force_mut(
        &mut self,
        key: &DirectedKey,
    ) -> &mut PositionCache {
        if !self.exists(key) {
            let pe = PositionCache::start(key.index.clone());
            if let Some(ve) = self.get_vertex_mut(&key.index) {
                ve.insert(&key.pos, pe);
            } else {
                let mut ve = VertexCache::from(key.index.clone());
                ve.insert(&key.pos, pe);
                self.entries.insert(key.index.vertex_index(), ve);
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
        self.get_vertex_mut(&key.index)
            .and_then(|ve| ve.get_mut(&key.pos))
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

impl Extend<(VertexIndex, VertexCache)> for TraceCache {
    fn extend<T: IntoIterator<Item = (VertexIndex, VertexCache)>>(
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
