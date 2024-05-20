use crate::shared::*;

mod entry;
pub use entry::*;
use seqraph::vertex::AsChild;

#[derive(Debug, Clone, Copy, From, new, Default, Deref, Hash, Eq, PartialEq)]
pub struct NGramId {
    #[deref]
    pub id: usize,
    pub width: usize,
}
impl Indexed for NGramId {
    fn vertex_index(&self) -> VertexIndex {
        self.id
    }
}
impl Wide for NGramId {
    fn width(&self) -> usize {
        self.width
    }
}

#[derive(Debug, Default)]
pub struct Vocabulary {
    //#[deref]
    pub graph: Hypergraph,
    pub ids: HashMap<String, NGramId>,
    pub labels: HashMap<VertexIndex, VocabEntry>,
    pub leaves: HashSet<VertexIndex>,
    pub roots: HashSet<VertexIndex>,
}

impl Vocabulary {
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    //pub fn clean(&mut self) -> HashSet<NGramId> {
    //    let drained: HashSet<_> = self.entries
    //        .extract_if(|_, e| !e.needs_node())
    //        .map(|(i, _)| i)
    //        .collect();
    //    self.ids.retain(|_, i| !drained.contains(i));
    //    drained
    //}
}

// define how to access a graph
// useful if you store extra labels for nodes by which to query
pub trait IndexVocab<K: ?Sized> {
    fn entry(&mut self, key: K) -> VertexEntry;
    fn get(&self, key: &K) -> Option<VertexCtx>;
    fn get_mut(&mut self, key: &K) -> Option<VertexCtxMut>;
}
#[derive(Debug, Deref, Clone)]
pub struct VertexCtx<'a> {
    pub data: &'a VertexData,
    #[deref]
    pub entry: &'a VocabEntry,
}
impl<'a> VertexCtx<'a> {
    pub fn direct_parents(&self) -> &HashMap<VertexIndex, Parent> {
        &self.data.parents
    }
}
#[derive(Debug, Deref)]
pub struct VertexCtxMut<'a> {
    pub data: &'a mut VertexData,
    #[deref]
    pub entry: &'a mut VocabEntry,
}
pub trait VocabIndex: Indexed {}
//impl VocabIndex for VertexIndex {}
//impl VocabIndex for NGramId {}
macro_rules! impl_index_vocab {
    ($t:ty) => {
        impl IndexVocab<$t> for Vocabulary {
            fn entry(&mut self, key: $t) -> VertexEntry {
                self.graph.vertex_entry(key)
            }
            fn get(&self, key: &$t) -> Option<VertexCtx> {
                self.graph.get_vertex_data(key).ok().map(|data| {
                    VertexCtx {
                        data,
                        entry: self.labels.get(key).unwrap(),
                    }
                })
            }
            fn get_mut(&mut self, key: &$t) -> Option<VertexCtxMut> {
                self.graph.get_vertex_data_mut(key).ok().map(|data| {
                    VertexCtxMut {
                        data,
                        entry: self.labels.get_mut(key).unwrap(),
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
        impl IndexVocab<$t> for Vocabulary {
            fn entry(&mut self, key: $t) -> VertexEntry {
                self.entry(*self.ids.get(key.borrow() as &'_ str).unwrap())
            }
            fn get(&self, key: &$t) -> Option<VertexCtx> {
                self.get(self.ids.get(key.borrow() as &'_ str)?)
            }
            fn get_mut(&mut self, key: &$t) -> Option<VertexCtxMut> {
                let id = *self.ids.get(key.borrow() as &'_ str)?;
                self.get_mut(&id)
            }
        }
    };
}
impl_index_vocab_str!(&'_ str);
impl_index_vocab_str!(String);
