use derive_more::Deref;
use serde::{
    Deserialize,
    Serialize,
};

use seqraph::{HashMap, HashSet};
use seqraph::graph::vertex::{data::VertexData, VertexEntry, IndexedVertexEntry, VertexIndex};
use seqraph::graph::vertex::has_vertex_index::HasVertexIndex;
use seqraph::graph::vertex::parent::Parent;

use crate::graph::containment::TextLocation;
use crate::graph::vocabulary::{NGramId, Vocabulary};
use std::borrow::Borrow;


// define how to access a graph
// useful if you store extra labels for nodes by which to query
pub trait HasVertexEntries<K: ?Sized>
{
    fn entry(
        &mut self,
        key: K,
    ) -> Option<IndexedVertexEntry<'_>>;
    fn get_vertex(
        &self,
        key: &K,
    ) -> Option<VertexCtx>;
    fn get_vertex_mut(
        &mut self,
        key: &K,
    ) -> Option<VertexCtxMut>;
}
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
pub trait VocabIndex: HasVertexIndex {}
//impl VocabIndex for VertexIndex {}
//impl VocabIndex for NGramId {}
macro_rules! impl_index_vocab {
    ($t:ty) => {
        impl HasVertexEntries<$t> for Vocabulary
        {
            fn entry(
                &mut self,
                key: $t,
            ) -> Option<IndexedVertexEntry<'_>>
            {
                self.containment.vertex_entry(key)
            }
            fn get_vertex(
                &self,
                index: &$t,
            ) -> Option<VertexCtx>
            {
                self.containment.get_vertex_data(index).ok().map(|data| {
                    VertexCtx {
                        data,
                        entry: self.entries.get(index).unwrap(),
                        vocab: self,
                    }
                })
            }
            fn get_vertex_mut(
                &mut self,
                index: &$t,
            ) -> Option<VertexCtxMut>
            {
                self.containment.get_vertex_data_mut(index).ok().map(|data| {
                    VertexCtxMut {
                        data,
                        entry: self.entries.get_mut(index).unwrap(),
                    }
                })
            }
        }
    };
}
impl_index_vocab!(VertexIndex);
impl_index_vocab!(NGramId);

macro_rules! impl_index_vocab_str {
    ($t:ty) => {
        impl HasVertexEntries<$t> for Vocabulary
        {
            fn entry(
                &mut self,
                key: $t,
            ) -> Option<IndexedVertexEntry<'_>>
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
