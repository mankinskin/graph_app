use std::path::{absolute, Path, PathBuf};

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
        entry::HasVertexEntries,
        Vocabulary,
    },
};

pub mod containment;
pub mod labelling;
pub mod partitions;
pub mod traversal;
pub mod vocabulary;

lazy_static::lazy_static! {
    pub static ref CORPUS_DIR: PathBuf = absolute(PathBuf::from_iter([".", "test", "cache"])).unwrap();
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
pub fn parse_corpus(corpus: Corpus) -> ParseResult {
    let mut image = LabellingCtx::from_corpus(&corpus);

    image.label_freq();
    image.label_wrap();
    let graph = image.label_part();
    ParseResult {
        graph,
        containment: image.vocab.containment,
        labels: image.labels,
    }
}