use crate::graph::{
    containment::{CorpusCtx, TextLevelCtx},
    traversal::{
        TopDown,
        TraversalPolicy,
    },
    vocabulary::entry::{
        HasVertexEntries,
        VocabEntry,
    },
};
use derive_more::{
    Deref,
    From,
};
use derive_new::new;
use itertools::Itertools;
use seqraph::{
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
};
use tap::Tap;

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
pub struct NGramId
{
    #[deref]
    pub key: VertexKey,
    pub width: usize,
}
impl HasVertexKey for NGramId
{
    fn vertex_key(&self) -> VertexKey
    {
        self.key
    }
}
impl Wide for NGramId
{
    fn width(&self) -> usize
    {
        self.width
    }
}

lazy_static::lazy_static! {
    pub static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "test", "cache"])).unwrap();
}
#[derive(Debug, Default, Deref, Serialize, Deserialize, new)]
pub struct Corpus
{
    pub name: String,
    #[deref]
    pub texts: Vec<String>,
}
impl Corpus
{
    pub fn target_file_path(&self) -> impl AsRef<Path>
    {
        CORPUS_DIR.join(&self.name)
    }
}
#[derive(
    Default, Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize,
)]
pub enum ProcessStatus
{
    #[default]
    Empty,
    Containment,
    Frequency,
    Wrappers,
    Partitions,
}
impl PartialOrd for ProcessStatus
{
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering>
    {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}
impl Ord for ProcessStatus
{
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        (*self as usize).cmp(&(*other as usize))
    }
}
#[derive(
    Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Deref,
)]
pub struct Vocabulary
{
    //#[deref]
    pub containment: Hypergraph,
    pub name: String,
    #[deref]
    pub ids: HashMap<String, NGramId>,
    pub leaves: HashSet<NGramId>,
    pub roots: HashSet<NGramId>,
    pub entries: HashMap<VertexKey, VocabEntry>,
}

impl Vocabulary
{
    pub fn from_corpus(corpus: &Corpus) -> Self
    {
        let mut vocab: Vocabulary = Default::default();
        vocab.name.clone_from(&corpus.name);
        CorpusCtx {
            corpus,
        }.on_corpus(&mut vocab);
        vocab
    }
    /// get sub-vertex at range relative to index
    pub fn get_vertex_subrange(
        &self,
        index: &VertexKey,
        range: Range<usize>,
    ) -> Child
    {
        let mut entry = self.get_vertex(index).unwrap();
        let mut wrap = 0..entry.len();
        assert!(wrap.start <= range.start && wrap.end >= range.end);

        while range != wrap
        {
            let next = TopDown::next_nodes(&entry)
                .into_iter()
                .map(
                    |(pos, c)| (c.vertex_key(), pos..pos + c.width()), //pos <= range.start || pos + c.width() >= range.end
                )
                .find_or_first(|(_, w)| {
                    w.start == range.start || w.end == range.end
                })
                .unwrap();

            entry = self.get_vertex(&next.0).unwrap();
            wrap = next.1;
        }

        entry.data.to_child()
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
