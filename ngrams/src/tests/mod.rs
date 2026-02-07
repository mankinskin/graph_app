pub(crate) mod count;

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
use crate::cancellation::Cancellation;

pub(crate) const OTTOS_MOPS_CORPUS: [&str; 4] = [
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
pub(crate) struct TestCorpus {
    pub(crate) image: LabellingImage,
    pub(crate) corpus: Corpus,
    pub(crate) roots_test: HashSet<String>,
    pub(crate) leaves_test: HashSet<String>,
}
impl TestCorpus {
    pub(crate) fn new(
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
    pub(crate) fn get_roots_test(&self) -> Vec<String> {
        self.roots_test.iter().cloned().sorted().collect()
    }
    pub(crate) fn get_leaves_test(&self) -> Vec<String> {
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
pub(crate) struct LabelTest {
    frequency: HashSet<String>,
    wrapper: HashSet<String>,
    partition: HashSet<String>,
}
impl LabelTest {
    pub(crate) fn validate(&self) {
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
pub(crate) struct TestCase {
    #[deref]
    #[deref_mut]
    ctx: LabellingCtx,
    labels: LabelTest,
}
impl TestCase {
    pub(crate) fn execute(&mut self) {
        // graph of all containment edges between n and n+1
        self.corpus.test_containment();
        self.label_freq().unwrap();

        if *self.status.pass() == ProcessStatus::Frequency {
            let ctx = LabelTestCtx::new(self.labels(), self);
            ctx.test_roots();
            ctx.test_leaves();

            ctx.test_freq();
        }

        self.label_wrap().unwrap();

        if *self.status.pass() == ProcessStatus::Wrappers {
            let ctx = LabelTestCtx::new(self.labels(), self);
            ctx.test_wrap();
        }

        self.label_part().unwrap();

        if *self.status.pass() == ProcessStatus::Partitions {
            let ctx = LabelTestCtx::new(self.labels(), self);
            ctx.test_part();
        }
    }
}
#[derive(Debug, new)]
pub(crate) struct LabelTestCtx<'a> {
    labels: &'a HashSet<VertexKey>,
    test: &'a TestCase,
}
impl<'a> LabelTestCtx<'a> {
    pub(crate) fn test_roots(&self) {
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
    pub(crate) fn test_leaves(&self) {
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
    pub(crate) fn get_frequency_test(&self) -> Vec<String> {
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
    pub(crate) fn get_wrapper_test(&self) -> Vec<String> {
        self.get_frequency_test()
            .iter()
            .chain(self.test.labels.wrapper.iter())
            .sorted()
            .cloned()
            .collect()
    }
    pub(crate) fn get_partition_test(&self) -> Vec<String> {
        self.get_wrapper_test()
            .iter()
            .chain(self.test.labels.partition.iter())
            .sorted()
            .cloned()
            .collect()
    }
    pub(crate) fn label_strings_set(&self) -> HashSet<String> {
        self.labels
            .iter()
            .map(|vi| self.test.vocab().get_vertex(vi).unwrap().ngram.clone())
            .collect()
    }
    pub(crate) fn test_freq(&self) {
        let label_strings = self.label_strings_set();
        let frequency_test = self.get_frequency_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            frequency_test,
        );
    }
    pub(crate) fn test_wrap(&self) {
        let label_strings = self.label_strings_set();
        let wrapper_test = self.get_wrapper_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            wrapper_test,
        );
    }
    pub(crate) fn test_part(&self) {
        let label_strings = self.label_strings_set();
        let partition_test = self.get_partition_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            partition_test,
        );
    }
}

#[test]
pub(crate) fn test_graph1() {
    let corpus = ["abab", "abcabc", "babc"];
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    TestCase {
        ctx: LabellingCtx::from_corpus(
            Corpus::new("abab_corpus".to_owned(), texts),
            Cancellation::None,
        ).unwrap(),
        labels: test_labels! {
            [
                "ab",
                "abc",
                "bab",
            ],
            [] as [&str; 0],
            [] as [&str; 0],
        },
    }
    .execute();
}

// too slow!
#[allow(unused)]
pub(crate) fn test_graph2() {
    let corpus = OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();

    TestCase {
        ctx: LabellingCtx::from_corpus(
            Corpus::new("ottos_mops".to_owned(), texts),
            Cancellation::None,
        ).unwrap(),
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

#[test]
pub(crate) fn test_parse_corpus() {
    use crate::graph::{
        parse_corpus,
        Status,
    };

    let corpus = ["hello", "world", "hello world"];
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    let corpus_name = "test_parse_corpus".to_owned();

    let status = StatusHandle::from(Status::new(texts.clone()));

    let result = parse_corpus(
        Corpus::new(corpus_name, texts),
        status,
        Cancellation::None,
    );

    assert!(result.is_ok(), "parse_corpus should succeed");

    let parse_result = result.unwrap();

    // Verify that the result contains expected data
    assert!(
        !parse_result.labels.is_empty(),
        "Should have some labels after parsing"
    );

    // Verify the graph has vertices
    assert!(
        parse_result.graph.vertex_count() > 0,
        "Graph should have vertices"
    );

    // Verify the containment graph has vertices
    assert!(
        parse_result.containment.vertex_count() > 0,
        "Containment graph should have vertices"
    );

    // Validate ALL vertices in the partition graph can produce string representations
    let graph_keys: Vec<_> = parse_result.graph.vertex_keys().collect();
    for key in &graph_keys {
        let s = parse_result.graph.vertex_key_string(key);
        assert!(
            !s.is_empty(),
            "Vertex {:?} should have non-empty string representation",
            key
        );
    }

    // Validate ALL vertices in the containment graph can produce string representations
    let containment_keys: Vec<_> =
        parse_result.containment.vertex_keys().collect();
    for key in &containment_keys {
        let s = parse_result.containment.vertex_key_string(key);
        assert!(!s.is_empty(), "Containment vertex {:?} should have non-empty string representation", key);
    }
}

#[test]
pub(crate) fn test_parse_corpus_aabbaabbaa() {
    use crate::graph::{
        parse_corpus,
        Status,
    };

    let texts = vec!["aabbaabbaa".to_string()];
    let corpus_name = "test_aabbaabbaa".to_owned();

    let status = StatusHandle::from(Status::new(texts.clone()));

    let result = parse_corpus(
        Corpus::new(corpus_name, texts),
        status,
        Cancellation::None,
    );

    assert!(result.is_ok(), "parse_corpus should succeed for aabbaabbaa");

    let parse_result = result.unwrap();

    // Verify basic structure
    assert!(
        parse_result.graph.vertex_count() > 0,
        "Graph should have vertices"
    );
    assert!(
        parse_result.containment.vertex_count() > 0,
        "Containment should have vertices"
    );

    // The string "aabbaabbaa" has repeating patterns like "aa", "bb", "aabb"
    // These should appear as frequency labels
    assert!(
        !parse_result.labels.is_empty(),
        "Should have frequency labels"
    );

    // Validate ALL vertices in the partition graph
    let graph_keys: Vec<_> = parse_result.graph.vertex_keys().collect();
    for key in &graph_keys {
        let s = parse_result.graph.vertex_key_string(key);
        assert!(
            !s.is_empty(),
            "Vertex {:?} should have non-empty string representation",
            key
        );
    }

    // Validate ALL vertices in the containment graph
    let containment_keys: Vec<_> =
        parse_result.containment.vertex_keys().collect();
    for key in &containment_keys {
        let s = parse_result.containment.vertex_key_string(key);
        assert!(!s.is_empty(), "Containment vertex {:?} should have non-empty string representation", key);
    }
}

#[test]
pub(crate) fn test_parse_corpus_single_char() {
    use crate::graph::{
        parse_corpus,
        Status,
    };

    let texts = vec!["aaaa".to_string()];
    let corpus_name = "test_single_char".to_owned();

    let status = StatusHandle::from(Status::new(texts.clone()));

    let result = parse_corpus(
        Corpus::new(corpus_name, texts),
        status,
        Cancellation::None,
    );

    assert!(result.is_ok(), "parse_corpus should succeed for aaaa");

    let parse_result = result.unwrap();
    assert!(
        parse_result.graph.vertex_count() > 0,
        "Graph should have vertices"
    );

    // "aaaa" contains "aa" 3 times, so it should be a frequency label
    assert!(
        !parse_result.labels.is_empty(),
        "Should have labels for repeated aa"
    );

    // Validate ALL vertices in the partition graph
    let graph_keys: Vec<_> = parse_result.graph.vertex_keys().collect();
    for key in &graph_keys {
        let s = parse_result.graph.vertex_key_string(key);
        assert!(
            !s.is_empty(),
            "Vertex {:?} should have non-empty string representation",
            key
        );
    }

    // Validate ALL vertices in the containment graph
    let containment_keys: Vec<_> =
        parse_result.containment.vertex_keys().collect();
    for key in &containment_keys {
        let s = parse_result.containment.vertex_key_string(key);
        assert!(!s.is_empty(), "Containment vertex {:?} should have non-empty string representation", key);
    }
}

#[test]
pub(crate) fn test_parse_corpus_two_texts() {
    use crate::graph::{
        parse_corpus,
        Status,
    };

    let texts = vec!["abc".to_string(), "bcd".to_string()];
    let corpus_name = "test_two_texts".to_owned();

    let status = StatusHandle::from(Status::new(texts.clone()));

    let result = parse_corpus(
        Corpus::new(corpus_name, texts),
        status,
        Cancellation::None,
    );

    assert!(result.is_ok(), "parse_corpus should succeed for two texts");

    let parse_result = result.unwrap();
    assert!(
        parse_result.graph.vertex_count() > 0,
        "Graph should have vertices"
    );

    // "bc" appears in both texts, so it should be labeled
    assert!(
        !parse_result.labels.is_empty(),
        "Should have labels for shared bc"
    );

    // Validate ALL vertices in the partition graph
    let graph_keys: Vec<_> = parse_result.graph.vertex_keys().collect();
    for key in &graph_keys {
        let s = parse_result.graph.vertex_key_string(key);
        assert!(
            !s.is_empty(),
            "Vertex {:?} should have non-empty string representation",
            key
        );
    }

    // Validate ALL vertices in the containment graph
    let containment_keys: Vec<_> =
        parse_result.containment.vertex_keys().collect();
    for key in &containment_keys {
        let s = parse_result.containment.vertex_key_string(key);
        assert!(!s.is_empty(), "Containment vertex {:?} should have non-empty string representation", key);
    }
}

#[test]
pub(crate) fn test_parse_corpus_empty_result() {
    use crate::graph::{
        parse_corpus,
        Status,
    };

    // A corpus with no repeated n-grams
    let texts = vec!["xyz".to_string()];
    let corpus_name = "test_no_repeats".to_owned();

    let status = StatusHandle::from(Status::new(texts.clone()));

    let result = parse_corpus(
        Corpus::new(corpus_name, texts),
        status,
        Cancellation::None,
    );

    assert!(
        result.is_ok(),
        "parse_corpus should succeed even with no repeats"
    );

    let parse_result = result.unwrap();
    // Should still have the basic structure
    assert!(
        parse_result.containment.vertex_count() > 0,
        "Containment should have vertices"
    );

    // Validate ALL vertices in the partition graph (may be empty or minimal)
    let graph_keys: Vec<_> = parse_result.graph.vertex_keys().collect();
    for key in &graph_keys {
        let s = parse_result.graph.vertex_key_string(key);
        assert!(
            !s.is_empty(),
            "Vertex {:?} should have non-empty string representation",
            key
        );
    }

    // Validate ALL vertices in the containment graph
    let containment_keys: Vec<_> =
        parse_result.containment.vertex_keys().collect();
    for key in &containment_keys {
        let s = parse_result.containment.vertex_key_string(key);
        assert!(!s.is_empty(), "Containment vertex {:?} should have non-empty string representation", key);
    }
}

#[test]
pub(crate) fn test_parse_corpus_empty_texts() {
    use crate::graph::{
        parse_corpus,
        Status,
        traversal::pass::CancelReason,
    };

    // Empty corpus (no texts)
    let texts: Vec<String> = vec![];
    let corpus_name = "test_empty_corpus".to_owned();

    let status = StatusHandle::from(Status::new(Vec::<String>::new()));

    let result = parse_corpus(
        Corpus::new(corpus_name, texts),
        status,
        Cancellation::None,
    );

    assert!(result.is_err(), "parse_corpus should fail for empty corpus");
    
    if let Err(CancelReason::EmptyVocabulary) = result {
        // Expected error
    } else {
        panic!("Expected EmptyVocabulary error, got: {:?}", result);
    }
}

#[test]
pub(crate) fn test_parse_corpus_only_empty_strings() {
    use crate::graph::{
        parse_corpus,
        Status,
        traversal::pass::CancelReason,
    };

    // Corpus with only empty strings
    let corpus_name = "test_empty_strings".to_owned();

    let status = StatusHandle::from(Status::new(Vec::<String>::new()));

    let result = parse_corpus(
        Corpus::new(corpus_name, vec!["".to_string(), "".to_string()]),
        status,
        Cancellation::None,
    );

    assert!(result.is_err(), "parse_corpus should fail for corpus with only empty strings");
    
    if let Err(CancelReason::EmptyVocabulary) = result {
        // Expected error
    } else {
        panic!("Expected EmptyVocabulary error, got: {:?}", result);
    }
}
