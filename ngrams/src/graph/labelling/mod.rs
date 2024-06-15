use std::cmp::Reverse;
use std::fs::{File, remove_file};
use std::io::{BufReader, BufWriter};
use std::path::{absolute, Path, PathBuf};
use plotters::style::text_anchor::VPos;
use seqraph::{
    vertex::VertexIndex,
    HashSet,
};
use ciborium::{
    de::Error as DeError,
    ser::Error as SerError,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tap::Tap;

use crate::graph::vocabulary::{Corpus, CORPUS_DIR, ProcessStatus, Vocabulary};

mod frequency;
use frequency::FrequencyCtx;

mod wrappers;
use crate::graph::partitions::PartitionsCtx;
use wrappers::WrapperCtx;
use crate::graph::LabelTestCtx;

impl From<Vocabulary> for LabellingCtx {
    fn from(vocab: Vocabulary) -> Self {
        Self {
            vocab,
            labels: Default::default(),
            status: ProcessStatus::Containment,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LabellingCtx
{
    pub vocab: Vocabulary,
    pub labels: HashSet<VertexIndex>,
    pub status: ProcessStatus,
}
impl LabellingCtx
{
    pub fn new(vocab: Vocabulary) -> Self
    {
        let mut ctx = Self {
            vocab,
            labels: Default::default(),
            status: ProcessStatus::Containment,
            //FromIterator::from_iter(
            //    vocab.leaves.iter().chain(
            //        vocab.roots.iter()
            //    )
            //    .cloned(),
            //),
        };
        ctx
    }
    pub fn target_file_path(&self) -> impl AsRef<Path>
    {
        CORPUS_DIR.join(&self.vocab.name)
    }
    pub fn write_to_target_file(
        &self,
    ) -> Result<(), SerError<std::io::Error>>
    {
        self.write_to_file(self.target_file_path())
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
    pub fn from_corpus(corpus: &Corpus) -> Self
    {
        Self::read_from_file(corpus.target_file_path())
            .inspect(|new| println!("Containment Pass already processed."))
            .unwrap_or_else(|e| {
                println!("{:#?}", e);
                Self::from(Vocabulary::from_corpus(corpus))
            })
    }
    pub fn label_freq(&mut self)
    {
        //let roots = texts.iter().map(|s| *vocab.ids.get(s).unwrap()).collect_vec();
        if (self.status < ProcessStatus::Frequency)
        {
            FrequencyCtx::from(&mut *self).frequency_pass();
            self.write_to_target_file();
        } else {
            println!("Frequency Pass already processed.");
        }
    }
    pub fn label_wrap(&mut self)
    {
        if (self.status < ProcessStatus::Wrappers)
        {
            WrapperCtx::from(&mut *self).wrapping_pass();
            //self.write_to_target_file();
        }
        else
        {
            println!("Wrapper Pass already processed.");
        }
    }
    pub fn label_part(&mut self)
    {
        //println!("{:#?}",
        //    self.vocab.entries.iter()
        //        .filter_map(|(i, e)|
        //            self.labels.contains(i).then(|| e.ngram.clone())
        //        )
        //        .sorted_by_key(|s| Reverse(s.len()))
        //        .collect_vec(),
        //);
        if (self.status < ProcessStatus::Partitions)
        {
            PartitionsCtx::from(&mut *self).partitions_pass();
            //self.write_to_target_file();
        }
        else
        {
            println!("Partition Pass already processed.");
        }
        //println!("{:#?}", vocab.labels);
    }
}
