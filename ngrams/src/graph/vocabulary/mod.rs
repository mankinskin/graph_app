use derive_more::{
    Deref,
    From,
};
use derive_new::new;

use crate::graph::containment::TextLevelCtx;
use seqraph::{
    graph::Hypergraph,
    HashMap,
    HashSet,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Range;
use std::path::{
    absolute,
    Path,
    PathBuf,
};
use itertools::Itertools;
use tap::Tap;
use seqraph::graph::vertex::{
    has_vertex_index::HasVertexIndex
    ,
    VertexIndex

    ,
    wide::Wide,
};
use seqraph::graph::vertex::child::Child;
use seqraph::graph::vertex::has_vertex_index::ToChild;
use seqraph::graph::vertex::key::VertexKey;
use crate::graph::traversal::{TopDown, TraversalPolicy};

use crate::graph::vocabulary::entry::{HasVertexEntries, VocabEntry};

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
    pub id: usize,
    pub width: usize,
}
impl HasVertexIndex for NGramId
{
    fn vertex_index(&self) -> VertexIndex
    {
        self.id
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
    pub static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "test", "corpus"])).unwrap();
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
    Default, Debug, PartialEq, Eq, Copy, Clone, Ord, Serialize, Deserialize,
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
        (*self as usize).partial_cmp(&(*other as usize))
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vocabulary
{
    //#[deref]
    pub containment: Hypergraph,
    pub name: String,
    pub ids: HashMap<String, NGramId>,
    pub leaves: HashSet<VertexKey>,
    pub roots: HashSet<VertexKey>,
    pub entries: HashMap<VertexKey, VocabEntry>,
}

impl Vocabulary
{
    pub fn len(&self) -> usize
    {
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
    pub fn from_corpus(corpus: &Corpus) -> Self
    {
        let mut vocab: Vocabulary = Default::default();
        vocab.name = corpus.name.clone();
        let N: usize = corpus.iter().map(|t| t.len()).max().unwrap();
        for n in 1..=N
        {
            let c = vocab.containment.vertex_count();
            for (i, text) in corpus.iter().enumerate()
            {
                TextLevelCtx::new(i, text, n).on_nlevel(&mut vocab);
            }
            //vocab.clean();
            //println!("Finished counting  n={}: (+{}) {}", n, vocab.len()-c, vocab.len())
        }
        vocab
    }
    /// get sub-vertex at range relative to index
    pub fn get_vertex_subrange(&self, index: &VertexIndex, range: Range<usize>) -> Child {
        let mut entry = self.get_vertex(index).unwrap();
        let mut wrap = 0..entry.len();
        assert!(wrap.start <= range.start && wrap.end >= range.end);

        while range != wrap {
            let next =
                TopDown::next_nodes(&entry)
                .into_iter()
                .map(|(pos, c)|
                         (c.vertex_index(), pos..pos + c.width())
                    //pos <= range.start || pos + c.width() >= range.end
                )
                .find_or_first(|(_, w)|
                    w.start == range.start || w.end == range.end
                )
                .unwrap();

            entry = self.get_vertex(&next.0).unwrap();
            wrap = next.1;
        }

        entry.data.to_child()
    }
}
