use derive_more::Deref;
use serde::{
    Deserialize,
    Serialize,
};

use seqraph::{HashMap, HashSet};
use seqraph::vertex::{VertexData, VertexEntry, VertexIndex};
use seqraph::vertex::indexed::Indexed;
use seqraph::vertex::parent::Parent;

use crate::graph::containment::TextLocation;
use crate::graph::vocabulary::{NGramId, Vocabulary};
use std::borrow::Borrow;


// define how to access a graph
// useful if you store extra labels for nodes by which to query
pub trait IndexVocab<K: ?Sized>
{
    fn entry(
        &mut self,
        key: K,
    ) -> VertexEntry;
    fn get(
        &self,
        key: &K,
    ) -> Option<VertexCtx>;
    fn get_mut(
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
pub trait VocabIndex: Indexed {}
//impl VocabIndex for VertexIndex {}
//impl VocabIndex for NGramId {}
macro_rules! impl_index_vocab {
    ($t:ty) => {
        impl IndexVocab<$t> for Vocabulary
        {
            fn entry(
                &mut self,
                key: $t,
            ) -> VertexEntry
            {
                self.graph.vertex_entry(key)
            }
            fn get(
                &self,
                key: &$t,
            ) -> Option<VertexCtx>
            {
                self.graph.get_vertex_data(key).ok().map(|data| {
                    VertexCtx {
                        data,
                        entry: self.entries.get(key).unwrap(),
                        vocab: self,
                    }
                })
            }
            fn get_mut(
                &mut self,
                key: &$t,
            ) -> Option<VertexCtxMut>
            {
                self.graph.get_vertex_data_mut(key).ok().map(|data| {
                    VertexCtxMut {
                        data,
                        entry: self.entries.get_mut(key).unwrap(),
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
        impl IndexVocab<$t> for Vocabulary
        {
            fn entry(
                &mut self,
                key: $t,
            ) -> VertexEntry
            {
                self.entry(*self.ids.get(key.borrow() as &'_ str).unwrap())
            }
            fn get(
                &self,
                key: &$t,
            ) -> Option<VertexCtx>
            {
                self.get(self.ids.get(key.borrow() as &'_ str)?)
            }
            fn get_mut(
                &mut self,
                key: &$t,
            ) -> Option<VertexCtxMut>
            {
                let id = *self.ids.get(key.borrow() as &'_ str)?;
                self.get_mut(&id)
            }
        }
    };
}
impl_index_vocab_str!(&'_ str);
impl_index_vocab_str!(String);
