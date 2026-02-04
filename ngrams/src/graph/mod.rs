use std::sync::{
    Arc,
    MappedRwLockReadGuard,
    MappedRwLockWriteGuard,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

#[cfg(not(target_arch = "wasm32"))]
use std::path::{
    absolute,
    PathBuf,
};

use derive_getters::Getters;
use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;
use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;

use context_trace::{
    graph::{
        getters,
        vertex::key::VertexKey,
        Hypergraph,
    },
    HashSet,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    cancellation::Cancellation,
    graph::{
        labelling::{
            LabellingCtx,
            LabellingImage,
        },
        traversal::pass::RunResult,
        vocabulary::{
            entry::HasVertexEntries,
            ProcessStatus,
            Vocabulary,
        },
    },
    tests::TestCorpus,
};

pub mod containment;
pub mod labelling;
pub mod partitions;
pub mod traversal;
pub mod utils;
pub mod vocabulary;

#[cfg(not(target_arch = "wasm32"))]
lazy_static::lazy_static! {
    pub static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "test", "cache"])).unwrap();
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct StatusHandle {
    data: Arc<RwLock<Status>>,
}
impl From<Status> for StatusHandle {
    fn from(value: Status) -> Self {
        Self {
            data: Arc::new(RwLock::new(value)),
        }
    }
}
impl StatusHandle {
    pub fn next_pass(
        &self,
        pass: ProcessStatus,
        steps: usize,
        steps_total: usize,
    ) {
        self.data
            .write()
            .unwrap()
            .next_pass(pass, steps, steps_total);
    }
    pub fn pass(&self) -> MappedRwLockReadGuard<'_, ProcessStatus> {
        RwLockReadGuard::<'_, Status>::map(self.data.read().unwrap(), |s| {
            &s.pass
        })
    }
    pub fn pass_mut(&self) -> MappedRwLockWriteGuard<'_, ProcessStatus> {
        RwLockWriteGuard::<'_, Status>::map(self.data.write().unwrap(), |s| {
            &mut s.pass
        })
    }
    pub fn steps(&self) -> MappedRwLockReadGuard<'_, usize> {
        RwLockReadGuard::<'_, Status>::map(self.data.read().unwrap(), |s| {
            &s.steps
        })
    }
    pub fn steps_mut(&self) -> MappedRwLockWriteGuard<'_, usize> {
        RwLockWriteGuard::<'_, Status>::map(self.data.write().unwrap(), |s| {
            &mut s.steps
        })
    }
    pub fn steps_total(&self) -> MappedRwLockReadGuard<'_, usize> {
        RwLockReadGuard::<'_, Status>::map(self.data.read().unwrap(), |s| {
            &s.steps_total
        })
    }
}
#[derive(Debug, Getters)]
pub struct Status {
    #[getter(skip)]
    pub insert_texts: Vec<String>,
    pass: ProcessStatus,
    steps: usize,
    steps_total: usize,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            insert_texts: Default::default(),
            pass: Default::default(),
            steps: 0,
            steps_total: 1,
        }
    }
}
impl Status {
    pub fn new(insert_texts: impl IntoIterator<Item = impl ToString>) -> Self {
        Self {
            insert_texts: insert_texts
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            ..Default::default()
        }
    }
    pub fn next_pass(
        &mut self,
        pass: ProcessStatus,
        steps: usize,
        steps_total: usize,
    ) {
        assert!(steps_total > 0);
        self.pass = pass;
        self.steps = steps;
        self.steps_total = steps_total.max(steps);
    }
}
#[derive(Debug, Default, Deref, Serialize, Deserialize)]
pub struct Corpus {
    pub name: String,
    #[deref]
    pub texts: Vec<String>,
}
impl Corpus {
    pub fn new(
        name: impl ToString,
        texts: impl IntoIterator<Item = impl ToString>,
    ) -> Self {
        let mut s = Status::new(Vec::<String>::new());
        s.pass = ProcessStatus::Frequency;
        Self {
            name: name.to_string(),
            texts: texts.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Get the storage key for this corpus
    pub fn storage_key(&self) -> &str {
        &self.name
    }
}
pub type AbortSender = std::sync::mpsc::Sender<()>;
pub type AbortReceiver = std::sync::mpsc::Receiver<()>;
pub struct ParseResult {
    pub graph: Hypergraph,
    pub containment: Hypergraph,
    pub labels: HashSet<VertexKey>,
}
pub fn parse_corpus(
    corpus: Corpus,
    mut status: StatusHandle,
    cancellation: impl Into<Cancellation>,
) -> RunResult<ParseResult> {
    let image = LabellingImage::from_corpus(&corpus, &mut status)?;
    let test_corpus = TestCorpus::new(image, corpus);
    let mut ctx = LabellingCtx::new(test_corpus, status, cancellation.into());

    ctx.label_freq()?;

    ctx.label_wrap()?;

    let graph = ctx.label_part()?;

    let LabellingImage { vocab, labels } = ctx.corpus.image;
    Ok(ParseResult {
        graph,
        labels,
        containment: vocab.containment,
    })
}
