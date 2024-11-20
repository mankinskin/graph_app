use std::{path::{absolute, Path, PathBuf}, sync::{Arc, MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard}};

use derive_new::new;
use derive_more::{DerefMut, Deref};
use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;
use derive_getters::Getters;

use seqraph::{graph::{getters, vertex::key::VertexKey, Hypergraph}, HashSet};
use serde::{Deserialize, Serialize};

use crate::graph::{
    labelling::LabellingCtx,
    vocabulary::{
        entry::HasVertexEntries, ProcessStatus, Vocabulary
    },
};

pub mod utils;
pub mod containment;
pub mod labelling;
pub mod partitions;
pub mod traversal;
pub mod vocabulary;

lazy_static::lazy_static! {
    pub static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "test", "cache"])).unwrap();
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct StatusHandle
{
    data: Arc<RwLock<Status>>,
}
impl From<Status> for StatusHandle {
    fn from(value: Status) -> Self {
        Self {
            data: Arc::new(RwLock::new(value)),
        }
    }
}
impl StatusHandle
{
    pub fn next_pass(&self, pass: ProcessStatus, steps: usize, steps_total: usize) {
        self.data.write().unwrap().next_pass(pass, steps, steps_total);
    }
    pub fn pass(&self) -> MappedRwLockReadGuard<'_, ProcessStatus> {
        RwLockReadGuard::<'_, Status>::map(self.data.read().unwrap(), |s| &s.pass)
    }
    pub fn pass_mut(&self) -> MappedRwLockWriteGuard<'_, ProcessStatus> {
        RwLockWriteGuard::<'_, Status>::map(self.data.write().unwrap(), |s| &mut s.pass)
    }
    pub fn steps(&self) -> MappedRwLockReadGuard<'_, usize> {
        RwLockReadGuard::<'_, Status>::map(self.data.read().unwrap(), |s| &s.steps)
    }
    pub fn steps_mut(&self) -> MappedRwLockWriteGuard<'_, usize> {
        RwLockWriteGuard::<'_, Status>::map(self.data.write().unwrap(), |s| &mut s.steps)
    }
    pub fn steps_total(&self) -> MappedRwLockReadGuard<'_, usize> {
        RwLockReadGuard::<'_, Status>::map(self.data.read().unwrap(), |s| &s.steps_total)
    }
}
#[derive(Debug, Getters)]
pub struct Status
{
    #[getter(skip)]
    pub insert_text: String,
    pass: ProcessStatus,
    steps: usize,
    steps_total: usize,
}
impl Default for Status
{
    fn default() -> Self {
        Self {
            insert_text: Default::default(),
            pass: Default::default(),
            steps: 0,
            steps_total: 1,
        }
    }
}
impl Status
{
    pub fn new(insert_text: impl ToString) -> Self {
        Self {
            insert_text: insert_text.to_string(),
            ..Default::default()
        }
    }
    pub fn next_pass(&mut self, pass: ProcessStatus, steps: usize, steps_total: usize) {
        assert!(steps_total > 0);
        self.pass = pass;
        self.steps = steps;
        self.steps_total = steps_total;
    }
}
#[derive(Debug, Default, Deref, Serialize, Deserialize)]
pub struct Corpus
{
    pub name: String,
    #[deref]
    pub texts: Vec<String>,
}
impl Corpus
{
    pub fn new(name: impl ToString, texts: impl IntoIterator<Item=impl ToString>) -> Self {
        let mut s = Status::new(String::new());
        s.pass = ProcessStatus::Frequency;
        Self {
            name: name.to_string(),
            texts: texts.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    pub fn target_file_path(&self) -> impl AsRef<Path>
    {
        CORPUS_DIR.join(&self.name)
    }
}
pub struct ParseResult {
    pub graph: Hypergraph,
    pub containment: Hypergraph,
    pub labels: HashSet<VertexKey>,
}
pub fn parse_corpus(corpus: Corpus, status: StatusHandle) -> ParseResult {
    let mut ctx = LabellingCtx::from_corpus(&corpus, status);

    ctx.label_freq();
    ctx.label_wrap();
    let graph = ctx.label_part();
    ParseResult {
        graph,
        containment: ctx.image.vocab.containment,
        labels: ctx.image.labels,
    }
}