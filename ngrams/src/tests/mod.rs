pub mod count;

use std::path::Path;

use crate::graph::{
    labelling::{
        frequency,
        LabellingCtx,
        LabellingImage,
    },
    vocabulary::{
        entry::HasVertexEntries,
        ProcessStatus,
        Vocabulary,
    },
    Corpus,
    StatusHandle,
};  
use context_trace::{
    graph::vertex::key::VertexKey,
    HashSet,
};
use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;
use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;
use tokio_util::sync::CancellationToken;

pub const OTTOS_MOPS_CORPUS: [&str; 4] = [
    "ottos mops trotzt",
    "otto: fort mops fort",
    "ottos mops hopst fort",
    "otto: soso",
];
fn read_corpus(file_path: impl AsRef<Path>) -> String {
    //let corpus: String = String::from("fldfjdlsjflskdjflsdfaädüwwrivfokl");
    let mut csv = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(file_path)
        .expect("Corpus file not found.");
    csv.records().map(|r| r.unwrap()[1].to_string()).join(" ")
}

#[derive(Debug)]
pub struct TestCorpus {
    pub image: LabellingImage,
    pub corpus: Corpus,
    pub roots_test: HashSet<String>,
    pub leaves_test: HashSet<String>,
}
impl TestCorpus {
    pub fn new(
        image: LabellingImage,
        corpus: Corpus,
    ) -> Self {
        let roots_test: HashSet<_> =
            corpus.texts.iter().map(ToString::to_string).collect();
        let leaves_test: HashSet<_> = corpus
            .texts
            .iter()
            .flat_map(|s| {
                s.chars().ngrams(1).map(String::from_iter).collect_vec()
            })
            .collect();
        Self {
            image,
            corpus,
            roots_test,
            leaves_test,
        }
    }
    pub fn get_roots_test(&self) -> Vec<String> {
        self.roots_test.iter().cloned().sorted().collect()
    }
    pub fn get_leaves_test(&self) -> Vec<String> {
        self.leaves_test.iter().cloned().sorted().collect()
    }
    pub(crate) fn test_containment(&self) {
        let Self {
            image: LabellingImage { vocab, .. },
            corpus,
            leaves_test,
            roots_test,
        } = self;
        assert_eq!(
            vocab
                .leaves
                .iter()
                .map(|vi| { vocab.get_vertex(vi).unwrap().ngram.clone() })
                .collect::<HashSet<_>>(),
            *leaves_test,
        );
        assert_eq!(
            vocab
                .roots
                .iter()
                .map(|vi| { vocab.get_vertex(vi).unwrap().ngram.clone() })
                .collect::<HashSet<_>>(),
            *roots_test,
        );
        for (k, e) in &vocab.entries {
            let patterns = vocab.containment.expect_child_patterns(k);
            assert!([0, 1, 2].contains(&patterns.len()));
            for (pid, p) in patterns.iter() {
                assert_eq!(
                    p.iter()
                        .map(|i| {
                            vocab
                                .entries
                                .get(&vocab.containment.expect_key_for_index(i))
                                .unwrap()
                                .ngram
                                .clone()
                        })
                        .join(""),
                    e.ngram,
                );
            }
        }
    }
}
#[derive(Debug, new)]
pub struct LabelTest {
    frequency: HashSet<String>,
    wrapper: HashSet<String>,
    partition: HashSet<String>,
}
impl LabelTest {
    pub fn validate(&self) {
        for (a, b) in [&self.frequency, &self.wrapper, &self.partition]
            .into_iter()
            .combinations(2)
            .map(|v| (v[0], v[1]))
        {
            assert_eq!(
                a.intersection(b).cloned().collect_vec(),
                Vec::<String>::default(),
            );
        }
    }
}
macro_rules! test_labels {
    ($freq:expr, $wrap:expr,$part:expr$(,)?) => {{
        let frequency = $freq.into_iter().map(ToString::to_string).collect();
        let wrapper: HashSet<String> =
            $wrap.into_iter().map(ToString::to_string).collect();
        let partition: HashSet<String> =
            $part.into_iter().map(ToString::to_string).collect();
        let s = LabelTest {
            frequency,
            wrapper,
            partition,
        };
        s.validate();
        s
    }};
}

#[derive(Debug, new, Deref, DerefMut)]
pub struct TestCase {
    #[deref]
    #[deref_mut]
    ctx: LabellingCtx,
    labels: LabelTest,
}
impl TestCase {
    pub fn execute(&mut self) {
        // graph of all containment edges between n and n+1
        self.corpus.test_containment();
        self.label_freq();

        if *self.status.pass() == ProcessStatus::Frequency {
            let ctx = LabelTestCtx::new(self.labels(), &self);
            ctx.test_roots();
            ctx.test_leaves();

            ctx.test_freq();
        }

        self.label_wrap();

        if *self.status.pass() == ProcessStatus::Wrappers {
            let ctx = LabelTestCtx::new(self.labels(), &self);
            ctx.test_wrap();
        }

        self.label_part();

        if *self.status.pass() == ProcessStatus::Partitions {
            let ctx = LabelTestCtx::new(self.labels(), &self);
            ctx.test_part();
        }
    }
}
#[derive(Debug, new)]
pub struct LabelTestCtx<'a> {
    labels: &'a HashSet<VertexKey>,
    test: &'a TestCase,
}
impl<'a> LabelTestCtx<'a> {
    pub fn test_roots(&self) {
        let label_strings = self.label_strings_set();
        let roots_test = self.test.corpus.get_roots_test();
        assert_eq!(
            label_strings
                .intersection(&roots_test.iter().cloned().collect())
                .cloned()
                .sorted()
                .collect_vec(),
            roots_test,
        );
    }
    pub fn test_leaves(&self) {
        let label_strings = self.label_strings_set();
        let leaves_test = self.test.corpus.get_leaves_test();
        assert_eq!(
            label_strings
                .intersection(&leaves_test.iter().cloned().collect())
                .cloned()
                .sorted()
                .collect_vec(),
            leaves_test,
        );
    }
    pub fn get_frequency_test(&self) -> Vec<String> {
        self.test
            .corpus
            .get_leaves_test()
            .iter()
            .chain(self.test.corpus.get_roots_test().iter())
            .chain(self.test.labels.frequency.iter())
            .sorted()
            .cloned()
            .collect()
    }
    pub fn get_wrapper_test(&self) -> Vec<String> {
        self.get_frequency_test()
            .iter()
            .chain(self.test.labels.wrapper.iter())
            .sorted()
            .cloned()
            .collect()
    }
    pub fn get_partition_test(&self) -> Vec<String> {
        self.get_wrapper_test()
            .iter()
            .chain(self.test.labels.partition.iter())
            .sorted()
            .cloned()
            .collect()
    }
    pub fn label_strings_set(&self) -> HashSet<String> {
        self.labels
            .iter()
            .map(|vi| self.test.vocab().get_vertex(vi).unwrap().ngram.clone())
            .collect()
    }
    pub fn test_freq(&self) {
        let label_strings = self.label_strings_set();
        let frequency_test = self.get_frequency_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            frequency_test,
        );
    }
    pub fn test_wrap(&self) {
        let label_strings = self.label_strings_set();
        let wrapper_test = self.get_wrapper_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            wrapper_test,
        );
    }
    pub fn test_part(&self) {
        let label_strings = self.label_strings_set();
        let partition_test = self.get_partition_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            partition_test,
        );
    }
}

#[tokio::test]
pub async fn test_graph1() {
    let corpus = ["abab", "abcabc", "babc"];
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    TestCase {
        ctx: LabellingCtx::from_corpus(
            Corpus::new("abab_corpus".to_owned(), texts),
            CancellationToken::new(),
        )
        .await,
        labels: test_labels! {
            [
                "ab"
            ],
            [
                "ab"
            ],
            [] as [&str; 0],
        },
    }
    .execute();
}

// too slow!
#[allow(unused)]
pub async fn test_graph2() {
    let corpus = OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();

    TestCase {
        ctx: LabellingCtx::from_corpus(
            Corpus::new("ottos_mops".to_owned(), texts),
            CancellationToken::new(),
        )
        .await,
        labels: test_labels! {
            [
                "ot",
                "s ",
                "so",
                "os",
                "t ",
                "ops",
                "otto",
                " mops ",
                "otto: ",
                " fort",
                "ottos mops ",
            ],
            [
                // Todo: check for correctness
                " fort ",
                " fort mops ",
                " fort mops fort",
                " mops fort",
                "ops ",
                "ops fort",
                "os ",
                "os mops ",
                "oso",
                "oso",
                "otto: fort",
                "otto: fort ",
                "otto: fort mops ",
                "ottos",
                "ottos ",
                "s fort",
                "s mops ",
                "sos",
                "soso",
                "t fort",
                "t mops ",
                "t mops fort",
            ],
            [] as [&str; 0],
        },
    }
    .execute();
}
