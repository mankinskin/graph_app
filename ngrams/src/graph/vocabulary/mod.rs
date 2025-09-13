use crate::graph::{
    containment::{
        CorpusCtx,
        TextLevelCtx,
    },
    traversal::direction::{
        TopDown,
        TraversalDirection,
    },
    vocabulary::entry::{
        HasVertexEntries,
        VocabEntry,
    },
    Corpus,
};
use context_trace::{
    graph::{
        vertex::{
            child::Child,
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            has_vertex_key::HasVertexKey,
            key::VertexKey,
            wide::Wide,
            VertexIndex,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
};
use derive_more::{
    Deref,
    From,
};
use derive_new::new;
use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::Display,
    ops::Range,
    path::{
        absolute,
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        RwLock,
    },
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tap::Tap;

use super::{
    Status,
    StatusHandle,
};

pub mod entry;

#[derive(
    Debug,
    Clone,
    Copy,
    From,
    new,
    Default,
    Deref,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub struct NGramId {
    #[deref]
    pub key: VertexKey,
    pub width: usize,
}
impl From<NGramId> for VertexKey {
    fn from(ngram_id: NGramId) -> Self {
        ngram_id.key
    }
}
impl HasVertexKey for NGramId {
    fn vertex_key(&self) -> VertexKey {
        self.key
    }
}
impl Wide for NGramId {
    fn width(&self) -> usize {
        self.width
    }
}

#[derive(
    Default, Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, EnumIter,
)]
pub enum ProcessStatus {
    #[default]
    Empty,
    Containment,
    Frequency,
    Wrappers,
    Partitions,
    Finished,
}
impl PartialOrd for ProcessStatus {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}
impl Ord for ProcessStatus {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}
#[derive(
    Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Deref,
)]
pub struct Vocabulary {
    //#[deref]
    pub containment: Hypergraph,
    pub name: String,
    #[deref]
    pub ids: HashMap<String, NGramId>,
    pub leaves: HashSet<NGramId>,
    pub roots: HashSet<NGramId>,
    pub entries: HashMap<VertexKey, VocabEntry>,
}

impl Vocabulary {
    pub fn from_corpus(
        corpus: &Corpus,
        status: &mut StatusHandle,
    ) -> Self {
        let mut vocab: Vocabulary = Default::default();
        vocab.name.clone_from(&corpus.name);
        CorpusCtx { corpus, status }.on_corpus(&mut vocab);
        vocab
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
