use derive_more::{
    Deref,
    From,
};
use derive_new::new;

use crate::graph::containment::TextLevelCtx;
use seqraph::{
    graph::Hypergraph,
    vertex::{
        indexed::Indexed,
        parent::Parent,
        wide::Wide,
        VertexData,
        VertexEntry,
        VertexIndex,
    },
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
use std::fs::{
    remove_file,
    File,
};
use std::io::{
    BufReader,
    BufWriter,
};
use std::path::{
    absolute,
    Path,
    PathBuf,
};
use tap::Tap;

use crate::graph::vocabulary::entry::VocabEntry;

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
impl Indexed for NGramId
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
    pub static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "corpus"])).unwrap();
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
    pub graph: Hypergraph,
    pub name: String,
    pub ids: HashMap<String, NGramId>,
    pub leaves: HashSet<VertexIndex>,
    pub roots: HashSet<VertexIndex>,
    pub entries: HashMap<VertexIndex, VocabEntry>,
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
            let c = vocab.graph.vertex_count();
            for (i, text) in corpus.iter().enumerate()
            {
                TextLevelCtx::new(i, text, n).on_nlevel(&mut vocab);
            }
            //vocab.clean();
            //println!("Finished counting  n={}: (+{}) {}", n, vocab.len()-c, vocab.len())
        }
        vocab
    }
}
