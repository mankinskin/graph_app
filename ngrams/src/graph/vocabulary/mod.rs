use derive_more::{
    Deref,
    From,
};
use derive_new::new;

use crate::graph::containment::TextLevelCtx;
use ciborium::{
    de::Error as DeError,
    ser::Error as SerError,
};
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
lazy_static::lazy_static! {
    static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "corpus"])).unwrap();
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
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vocabulary
{
    //#[deref]
    pub graph: Hypergraph,
    pub name: String,
    pub ids: HashMap<String, NGramId>,
    pub entries: HashMap<VertexIndex, VocabEntry>,
    pub labels: HashSet<VertexIndex>,
    pub leaves: HashSet<VertexIndex>,
    pub roots: HashSet<VertexIndex>,
    pub status: ProcessStatus,
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
    fn build_from_corpus(corpus: &Corpus) -> Self
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
        vocab.status = ProcessStatus::Containment;
        vocab
    }
    pub fn target_file_path(&self) -> impl AsRef<Path>
    {
        CORPUS_DIR.join(&self.name)
    }
    pub fn from_corpus(corpus: &Corpus) -> Self
    {
        Self::read_from_file(corpus.target_file_path())
            .inspect(|new| println!("Containment Pass already processed."))
            .unwrap_or_else(|e| {
                println!("{:#?}", e);
                Self::build_from_corpus(corpus)
            })
    }
    pub fn write_to_file(
        &self,
        file_path: impl AsRef<Path>,
    ) -> Result<(), SerError<std::io::Error>>
    {
        println!("Write Vocabulary to {}", file_path.as_ref().display());
        if file_path.as_ref().exists()
        {
            remove_file(&file_path);
        }
        let file = File::create(file_path).map_err(|e| SerError::Io(e))?;
        let mut writer = BufWriter::new(file);
        ciborium::into_writer(&self, writer)
    }
    pub fn read_from_file(
        file_path: impl AsRef<Path>
    ) -> Result<Self, DeError<std::io::Error>>
    {
        println!("Read Vocabulary from {}", file_path.as_ref().display());
        let file = File::open(file_path).map_err(|e| DeError::Io(e))?;
        let mut reader = BufReader::new(file);
        ciborium::from_reader(reader)
    }
}

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
