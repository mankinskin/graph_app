use derive_more::Deref;
use serde::{
    Deserialize,
    Serialize,
};

use context_trace::{
    graph::vertex::{
        data::VertexData,
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        key::VertexKey,
        parent::Parent,
        token::{
            Token,
            TokenWidth,
        },
        wide::Wide,
        VertexEntry,
        VertexIndex,
    },
    HashMap,
    HashSet,
    VertexSet,
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
pub(crate) struct VocabEntry {
    //pub(crate) id: NGramId,
    pub(crate) occurrences: HashSet<TextLocation>,
    // positions of largest smaller ngrams
    //pub(crate) children: NodeChildren,
    #[deref]
    pub(crate) ngram: String,
}

impl VocabEntry {
    pub(crate) fn count(&self) -> usize {
        self.occurrences.len()
    }
    //pub(crate) fn needs_node(&self) -> bool {
    //    self.len() == 1
    //        || self.children.has_overlaps()
    //}
}
#[derive(Debug, Deref, Clone)]
pub(crate) struct VertexCtx<'a> {
    pub(crate) data: VertexData,
    #[deref]
    pub(crate) entry: &'a VocabEntry,
    pub(crate) vocab: &'a Vocabulary,
}
impl Wide for VertexCtx<'_> {
    fn width(&self) -> TokenWidth {
        self.data.width()
    }
}
impl HasVertexKey for VertexCtx<'_> {
    fn vertex_key(&self) -> VertexKey {
        self.data.vertex_key()
    }
}
#[derive(Debug, Deref)]
pub(crate) struct VertexCtxMut<'a> {
    pub(crate) data: VertexData,
    #[deref]
    pub(crate) entry: &'a mut VocabEntry,
}
// define how to access a graph
// useful if you store extra labels for nodes by which to query
pub(crate) trait HasVertexEntries<K: ?Sized + Debug> {
    fn entry(
        &mut self,
        key: K,
    ) -> VertexEntry;
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
pub(crate) trait VocabIndex: HasVertexIndex {}
//impl VocabIndex for VertexIndex {}
//impl VocabIndex for NGramId {}
macro_rules! impl_index_vocab {
    ($t:ty, ($_self:ident, $ind:ident) => $func:expr) => {
        impl HasVertexEntries<$t> for Vocabulary
        {
            fn entry(
                &'_ mut $_self,
                $ind: $t,
            ) -> VertexEntry
            {
                let key = $func;
                let data = $_self.containment.expect_vertex_data(key);
                VertexEntry::new(data)
            }
            fn get_vertex(
                &'_ $_self,
                $ind: &$t,
            ) -> Option<VertexCtx<'_>>
            {
                let key = $func;
                $_self.containment.get_vertex_data(key).ok().map(|data| {
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
                $_self.containment.get_vertex_data(key).ok().map(|data| {
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
impl_index_vocab!(Token, (self, index) => self.containment.expect_key_for_index(index.vertex_index()));
impl_index_vocab!(VertexIndex, (self, index) => self.containment.expect_key_for_index(index));

macro_rules! impl_index_vocab_str {
    ($t:ty) => {
        impl HasVertexEntries<$t> for Vocabulary {
            fn entry(
                &'_ mut self,
                key: $t,
            ) -> VertexEntry {
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
