use std::{path::{absolute, Path, PathBuf}, sync::{Arc, RwLock}};

use derive_new::new;
use derive_more::Deref;
use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;

use seqraph::{graph::{vertex::key::VertexKey, Hypergraph}, HashSet};
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

#[derive(Debug)]
pub struct Status
{
    pub insert_text: String,
    pub pass: ProcessStatus,
}
impl Status
{
    pub fn new(insert_text: impl ToString) -> Self {
        Self {
            insert_text: insert_text.to_string(),
            pass: ProcessStatus::default(),
        }
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
pub fn parse_corpus(corpus: Corpus, status: Arc<RwLock<Status>>) -> ParseResult {
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