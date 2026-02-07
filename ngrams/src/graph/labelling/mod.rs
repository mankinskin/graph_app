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
use std::sync::{
    Arc,
    RwLock,
};
use tap::Tap;

use crate::{
    cancellation::{
        Cancellable,
        Cancellation,
    },
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
    },
    storage::{
        self,
        Storage,
        StorageError,
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

pub(crate) mod frequency;
use frequency::FrequencyCtx;

pub(crate) mod wrapper;
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
pub(crate) struct LabellingImage {
    pub(crate) vocab: Vocabulary,
    pub(crate) labels: HashSet<VertexKey>,
}
impl LabellingImage {
    /// Get the storage key for this labelling image
    pub(crate) fn storage_key(&self) -> String {
        self.vocab.name.clone()
    }

    /// Write to storage using the platform-appropriate storage backend
    pub(crate) fn write_to_storage(&self) -> Result<(), StorageError> {
        storage::store(&self.storage_key(), self)
    }

    /// Read from storage using the platform-appropriate storage backend
    pub(crate) fn read_from_storage(key: &str) -> Result<Self, StorageError> {
        storage::load(key)
    }

    /// Check if this labelling image exists in storage
    pub(crate) fn exists_in_storage(key: &str) -> bool {
        storage::exists(key)
    }

    pub(crate) fn from_corpus(
        corpus: &Corpus,
        status: &mut StatusHandle,
    ) -> RunResult<Self> {
        // On native, try to read from storage cache first
        #[cfg(not(target_arch = "wasm32"))]
        {
            let key = corpus.name.clone();
            if let Ok(image) = Self::read_from_storage(&key) {
                println!("Containment Pass already processed.");
                return Ok(image);
            }
        }

        // Create fresh from corpus
        Ok(Self::from(Vocabulary::from_corpus(corpus, status)?))
    }

    /// Write to the target storage location for this image
    pub(crate) fn write_to_target_file(&self) -> Result<(), StorageError> {
        self.write_to_storage()
    }
}
#[derive(Debug, Deref, DerefMut, new)]
pub(crate) struct LabellingCtx {
    #[deref]
    #[deref_mut]
    pub(crate) corpus: TestCorpus,
    pub(crate) status: StatusHandle,
    pub(crate) cancellation: Cancellation,
}
impl LabellingCtx {
    pub(crate) fn from_corpus(
        corpus: Corpus,
        cancellation: impl Into<Cancellation>,
    ) -> RunResult<Self> {
        let mut status = StatusHandle::default();
        Ok(Self {
            corpus: TestCorpus::new(
                LabellingImage::from_corpus(&corpus, &mut status)?,
                corpus,
            ),
            status,
            cancellation: cancellation.into(),
        })
    }
    pub(crate) fn check_cancelled(&self) -> RunResult<()> {
        if self.cancellation.is_cancelled() {
            Err(CancelReason::Cancelled)
        } else {
            Ok(())
        }
    }
    pub(crate) fn vocab(&self) -> &'_ Vocabulary {
        &self.corpus.image.vocab
    }
    pub(crate) fn labels(&self) -> &'_ HashSet<VertexKey> {
        &self.corpus.image.labels
    }
    pub(crate) fn labels_mut(&mut self) -> &'_ mut HashSet<VertexKey> {
        &mut self.corpus.image.labels
    }
    pub(crate) fn label_freq(&mut self) -> RunResult<()> {
        if *self.status.pass() < ProcessStatus::Frequency {
            FrequencyCtx::new(&mut *self).run()?;
            let _ = self.image.write_to_target_file();
        } else {
            println!("Frequency Pass already processed.");
        }
        Ok(())
    }
    pub(crate) fn label_wrap(&mut self) -> RunResult<()> {
        if *self.status.pass() < ProcessStatus::Wrappers {
            WrapperCtx::new(&mut *self).run()?;
            let _ = self.image.write_to_target_file();
        } else {
            println!("Wrapper Pass already processed.");
        }
        Ok(())
    }
    pub(crate) fn label_part(&mut self) -> RunResult<Hypergraph> {
        let mut ctx = PartitionsCtx::from(&mut *self);
        ctx.run()?;
        Ok(ctx.graph)
    }
}
