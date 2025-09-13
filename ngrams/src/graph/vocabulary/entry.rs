use context_trace::graph::vertex::wide::Wide;
use derive_more::Deref;
use serde::{
    Deserialize,
    Serialize,
};

use context_trace::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            data::VertexData,
            has_vertex_index::HasVertexIndex,
            has_vertex_key::HasVertexKey,
            key::VertexKey,
            parent::Parent,
            IndexedVertexEntry,
            VertexEntry,
            VertexIndex,
        },
    },
    HashMap,
    HashSet,
};

use crate::graph::{
    containment::TextLocation,
    vocabulary::{
        NGramId,
        Vocabulary,
    },
};
use std::{
    borrow::Borrow,
    fmt::Debug,
};

#[derive(Debug, Deref, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabEntry {
    //pub id: NGramId,
    pub occurrences: HashSet<TextLocation>,
    // positions of largest smaller ngrams
    //pub children: NodeChildren,
    #[deref]
    pub ngram: String,
}

impl VocabEntry {
    pub fn count(&self) -> usize {
        self.occurrences.len()
    }
    //pub fn needs_node(&self) -> bool {
    //    self.len() == 1
    //        || self.children.has_overlaps()
    //}
}
#[derive(Debug, Deref)]
pub struct VertexCtx<'a> {
    pub data: &'a VertexData,
    #[deref]
    pub entry: &'a VocabEntry,
    pub vocab: &'a Vocabulary,
}
impl Wide for VertexCtx<'_> {
    fn width(&self) -> usize {
        self.data.width()
    }
}
impl HasVertexKey for VertexCtx<'_> {
    fn vertex_key(&self) -> VertexKey {
        self.data.vertex_key()
    }
}
#[derive(Debug, Deref)]
pub struct VertexCtxMut<'a> {
    pub data: &'a mut VertexData,
    #[deref]
    pub entry: &'a mut VocabEntry,
}
// define how to access a graph
// useful if you store extra labels for nodes by which to query
pub trait HasVertexEntries<K: ?Sized + Debug> {
    fn entry(
        &mut self,
        key: K,
    ) -> VertexEntry<'_>;
    fn get_vertex(
        &'_ self,
        key: &K,
    ) -> Option<VertexCtx<'_>>;
    fn get_vertex_mut(
        &'_ mut self,
        key: &K,
    ) -> Option<VertexCtxMut<'_>>;
    fn expect_vertex(
        &'_ self,
        key: &K,
    ) -> VertexCtx<'_> {
        self.get_vertex(key)
            .unwrap_or_else(|| panic!("No VertexKey: {:?}", key))
    }
    fn expect_vertex_mut(
        &'_ mut self,
        key: &K,
    ) -> VertexCtxMut<'_> {
        self.get_vertex_mut(key)
            .unwrap_or_else(|| panic!("No VertexKey: {:?}", key))
    }
}
pub trait VocabIndex: HasVertexIndex {}
//impl VocabIndex for VertexIndex {}
//impl VocabIndex for NGramId {}
macro_rules! impl_index_vocab {
    ($t:ty, ($_self:ident, $ind:ident) => $func:expr) => {
        impl HasVertexEntries<$t> for Vocabulary
        {
            fn entry(
                &'_ mut $_self,
                $ind: $t,
            ) -> VertexEntry<'_>
            {
                $_self.containment.vertex_entry($func)
            }
            fn get_vertex(
                &'_ $_self,
                $ind: &$t,
            ) -> Option<VertexCtx<'_>>
            {
                let key = $func;
                $_self.containment.get_vertex(&key).ok().map(|data| {
                    VertexCtx {
                        data,
                        entry: $_self.entries.get(&key).unwrap(),
                        vocab: $_self,
                    }
                })
            }
            fn get_vertex_mut(
                &'_ mut $_self,
                $ind: &$t,
            ) -> Option<VertexCtxMut<'_>>
            {
                let key = $func;
                $_self.containment.get_vertex_mut(&key).ok().map(|data| {
                    VertexCtxMut {
                        data,
                        entry: $_self.entries.get_mut(&key).unwrap(),
                    }
                })
            }
        }
    };
}
impl_index_vocab!(VertexKey, (self, key) => key.vertex_key());
impl_index_vocab!(NGramId, (self, key) => key.vertex_key());
impl_index_vocab!(Child, (self, index) => self.containment.expect_key_for_index(index.vertex_index()));
impl_index_vocab!(VertexIndex, (self, index) => self.containment.expect_key_for_index(index));

macro_rules! impl_index_vocab_str {
    ($t:ty) => {
        impl HasVertexEntries<$t> for Vocabulary {
            fn entry(
                &'_ mut self,
                key: $t,
            ) -> VertexEntry<'_> {
                self.entry(*self.ids.get(key.borrow() as &'_ str).unwrap())
            }
            fn get_vertex(
                &'_ self,
                key: &$t,
            ) -> Option<VertexCtx<'_>> {
                self.get_vertex(self.ids.get(key.borrow() as &'_ str)?)
            }
            fn get_vertex_mut(
                &'_ mut self,
                key: &$t,
            ) -> Option<VertexCtxMut<'_>> {
                let id = *self.ids.get(key.borrow() as &'_ str)?;
                self.get_vertex_mut(&id)
            }
        }
    };
}
impl_index_vocab_str!(&'_ str);
impl_index_vocab_str!(String);
