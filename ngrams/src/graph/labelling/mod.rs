use ciborium::{
    de::Error as DeError,
    ser::Error as SerError,
};
use derive_more::derive::{
    Deref,
    DerefMut,
};
use derive_new::new;
use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs::{
        remove_file,
        File,
    },
    io::{
        BufReader,
        BufWriter,
    },
    path::Path,
    sync::{
        Arc,
        RwLock,
    },
};
use tap::Tap;
use tokio_util::sync::CancellationToken;

use crate::{
    graph::{
        partitions::PartitionsCtx,
        traversal::pass::{
            CancelReason,
            RunResult,
            TraversalPass,
        },
        vocabulary::{
            ProcessStatus,
            Vocabulary,
        },
        Corpus,
        Status,
        CORPUS_DIR,
    },
    tests::TestCorpus,
};
use context_trace::{
    graph::{
        vertex::{
            key::VertexKey,
            VertexIndex,
        },
        Hypergraph,
    },
    HashSet,
};

pub mod frequency;
use frequency::FrequencyCtx;

pub mod wrapper;
use wrapper::WrapperCtx;

use super::StatusHandle;

impl From<Vocabulary> for LabellingImage {
    fn from(vocab: Vocabulary) -> Self {
        Self {
            vocab,
            labels: Default::default(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LabellingImage {
    pub vocab: Vocabulary,
    pub labels: HashSet<VertexKey>,
}
impl LabellingImage {
    pub fn target_file_path(&self) -> impl AsRef<Path> {
        CORPUS_DIR.join(&self.vocab.name)
    }
    pub fn write_to_target_file(&self) -> Result<(), SerError<std::io::Error>> {
        self.write_to_file(self.target_file_path())
    }
    pub fn write_to_file(
        &self,
        file_path: impl AsRef<Path>,
    ) -> Result<(), SerError<std::io::Error>> {
        println!("Write Vocabulary to {}", file_path.as_ref().display());
        if file_path.as_ref().exists() {
            remove_file(&file_path);
        }
        std::fs::create_dir_all(file_path.as_ref().with_file_name(""));
        let file = File::create(file_path).map_err(SerError::Io)?;
        let mut writer = BufWriter::new(file);
        ciborium::into_writer(&self, writer)
    }
    pub fn read_from_file(
        file_path: impl AsRef<Path>
    ) -> Result<Self, DeError<std::io::Error>> {
        println!("Read Vocabulary from {}", file_path.as_ref().display());
        let file = File::open(file_path).map_err(DeError::Io)?;
        let mut reader = BufReader::new(file);
        ciborium::from_reader(reader)
    }
    pub async fn from_corpus(
        corpus: &Corpus,
        status: &mut StatusHandle,
    ) -> Self {
        Self::read_from_file(corpus.target_file_path())
            .inspect(|new| println!("Containment Pass already processed."))
            .unwrap_or_else(|e| {
                println!("{:#?}", e);
                Self::from(Vocabulary::from_corpus(corpus, status))
            })
    }
}
#[derive(Debug, Deref, DerefMut, new)]
pub struct LabellingCtx {
    #[deref]
    #[deref_mut]
    pub corpus: TestCorpus,
    pub status: StatusHandle,
    pub cancellation_token: CancellationToken,
}
impl LabellingCtx {
    pub async fn from_corpus(
        corpus: Corpus,
        cancellation_token: CancellationToken,
    ) -> Self {
        let mut status = StatusHandle::default();
        Self {
            corpus: TestCorpus::new(
                LabellingImage::from_corpus(&corpus, &mut status).await,
                corpus,
            ),
            status,
            cancellation_token,
        }
    }
    pub fn check_cancelled(&self) -> RunResult<()> {
        if self.cancellation_token.is_cancelled() {
            Err(CancelReason::Cancelled)
        } else {
            Ok(())
        }
    }
    pub fn vocab(&self) -> &'_ Vocabulary {
        &self.corpus.image.vocab
    }
    pub fn labels(&self) -> &'_ HashSet<VertexKey> {
        &self.corpus.image.labels
    }
    pub fn labels_mut(&mut self) -> &'_ mut HashSet<VertexKey> {
        &mut self.corpus.image.labels
    }
    pub async fn label_freq(&mut self) -> RunResult<()> {
        //let roots = texts.iter().map(|s| *vocab.ids.get(s).unwrap()).collect_vec();
        if *self.status.pass() < ProcessStatus::Frequency {
            FrequencyCtx::new(&mut *self).run()?;
            self.image.write_to_target_file();
        } else {
            println!("Frequency Pass already processed.");
        }
        Ok(())
    }
    pub async fn label_wrap(&mut self) -> RunResult<()> {
        if *self.status.pass() < ProcessStatus::Wrappers {
            WrapperCtx::new(&mut *self).run()?;
            self.image.write_to_target_file();
        } else {
            println!("Wrapper Pass already processed.");
        }
        Ok(())
    }
    pub async fn label_part(&mut self) -> RunResult<Hypergraph> {
        //println!("{:#?}",
        //    self.vocab.entries.iter()
        //        .filter_map(|(i, e)|
        //            self.labels.contains(i).then(|| e.ngram.clone())
        //        )
        //        .sorted_by_key(|s| Reverse(s.len()))
        //        .collect_vec(),
        //);
        let mut ctx = PartitionsCtx::from(&mut *self);
        ctx.run()?;
        Ok(ctx.graph)
    }
}
