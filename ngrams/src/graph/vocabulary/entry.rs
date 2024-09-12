use derive_more::Deref;
use serde::{
    Deserialize,
    Serialize,
};

use seqraph::graph::{
    getters::vertex::VertexSet,
    vertex::has_vertex_key::HasVertexKey,
};
use seqraph::{
    graph::vertex::{
        child::Child,
        data::VertexData,
        has_vertex_index::HasVertexIndex,
        key::VertexKey,
        parent::Parent,
        IndexedVertexEntry,
        VertexEntry,
        VertexIndex,
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
use std::borrow::Borrow;

#[derive(Debug, Deref, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabEntry
{
    //pub id: NGramId,
    pub occurrences: HashSet<TextLocation>,
    // positions of largest smaller ngrams
    //pub children: NodeChildren,
    #[deref]
    pub ngram: String,
}

impl VocabEntry
{
    pub fn count(&self) -> usize
    {
        self.occurrences.len()
    }
    //pub fn needs_node(&self) -> bool {
    //    self.len() == 1
    //        || self.children.has_overlaps()
    //}
}
#[derive(Debug, Deref)]
pub struct VertexCtx<'a>
{
    pub data: &'a VertexData,
    #[deref]
    pub entry: &'a VocabEntry,
    pub vocab: &'a Vocabulary,
}
impl<'a> VertexCtx<'a>
{
    pub fn direct_parents(&self) -> &HashMap<VertexIndex, Parent>
    {
        &self.data.parents
    }
}
#[derive(Debug, Deref)]
pub struct VertexCtxMut<'a>
{
    pub data: &'a mut VertexData,
    #[deref]
    pub entry: &'a mut VocabEntry,
}
// define how to access a graph
// useful if you store extra labels for nodes by which to query
pub trait HasVertexEntries<K: ?Sized>
{
    fn entry(
        &mut self,
        key: K,
    ) -> VertexEntry<'_>;
    fn get_vertex(
        &self,
        key: &K,
    ) -> Option<VertexCtx>;
    fn get_vertex_mut(
        &mut self,
        key: &K,
    ) -> Option<VertexCtxMut>;
}
pub trait VocabIndex: HasVertexIndex {}
//impl VocabIndex for VertexIndex {}
//impl VocabIndex for NGramId {}
macro_rules! impl_index_vocab {
    ($t:ty, ($_self:ident, $ind:ident) => $func:expr) => {
        impl HasVertexEntries<$t> for Vocabulary
        {
            fn entry(
                &mut $_self,
                $ind: $t,
            ) -> VertexEntry<'_>
            {
                $_self.containment.vertex_entry($func)
            }
            fn get_vertex(
                &$_self,
                $ind: &$t,
            ) -> Option<VertexCtx>
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
                &mut $_self,
                $ind: &$t,
            ) -> Option<VertexCtxMut>
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
        impl HasVertexEntries<$t> for Vocabulary
        {
            fn entry(
                &mut self,
                key: $t,
            ) -> VertexEntry<'_>
            {
                self.entry(*self.ids.get(key.borrow() as &'_ str).unwrap())
            }
            fn get_vertex(
                &self,
                key: &$t,
            ) -> Option<VertexCtx>
            {
                self.get_vertex(self.ids.get(key.borrow() as &'_ str)?)
            }
            fn get_vertex_mut(
                &mut self,
                key: &$t,
            ) -> Option<VertexCtxMut>
            {
                let id = *self.ids.get(key.borrow() as &'_ str)?;
                self.get_vertex_mut(&id)
            }
        }
    };
}
impl_index_vocab_str!(&'_ str);
impl_index_vocab_str!(String);
