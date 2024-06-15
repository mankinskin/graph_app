use derive_new::new;
use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;

use seqraph::HashSet;

use crate::graph::vocabulary::Corpus;
use crate::graph::{
    vocabulary::{
        entry::IndexVocab,
        Vocabulary,
    },
};
use crate::graph::labelling::{LabellingCtx};

mod containment;
mod labelling;
mod partitions;
mod traversal;
mod vocabulary;

#[derive(Debug)]
struct TestCtx<'a>
{
    vocab: &'a Vocabulary,
    corpus: &'a Corpus,
    roots_test: HashSet<String>,
    leaves_test: HashSet<String>,
    //labels: Option<&'a HashSet<usize>>,
}
impl<'a> TestCtx<'a>
{
    pub fn new(
        vocab: &'a Vocabulary,
        corpus: &'a Corpus,
        //labels: Option<&'a HashSet<usize>>,
    ) -> Self
    {
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
            vocab,
            corpus,
            roots_test,
            leaves_test,
        }
    }
    pub fn get_roots_test(&self) -> Vec<String>
    {
        self.roots_test.iter().cloned().sorted().collect()
    }
    pub fn get_leaves_test(&self) -> Vec<String>
    {
        self.leaves_test.iter().cloned().sorted().collect()
    }

    fn test_containment(&self)
    {
        let Self {
            vocab,
            corpus,
            leaves_test,
            roots_test,
            ..
        } = self;
        assert_eq!(
            vocab
                .leaves
                .iter()
                .map(|vi| { vocab.get(vi).unwrap().ngram.clone() })
                .collect::<HashSet<_>>(),
            *leaves_test,
        );
        assert_eq!(
            vocab
                .roots
                .iter()
                .map(|vi| { vocab.get(vi).unwrap().ngram.clone() })
                .collect::<HashSet<_>>(),
            *roots_test,
        );
    }
}
#[derive(Debug)]
struct LabelTestCtx<'a>
{
    ctx: TestCtx<'a>,
    labels: &'a HashSet<usize>,
    frequency_test: HashSet<String>,
    wrapper_test: HashSet<String>,
    partition_test: HashSet<String>,
}
impl<'a> LabelTestCtx<'a> {
    pub fn new(
        ctx: TestCtx<'a>,
        labels: &'a HashSet<usize>,
    ) -> Self {
        let frequency_test: HashSet<_> =
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
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect();
        let wrapper_test: HashSet<_> =
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
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect();
        let partition_test: HashSet<_> =
            (&[
            ] as &[&'static str])
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            frequency_test.intersection(&wrapper_test).cloned().collect_vec(),
            Vec::<String>::default(),
        );
        assert_eq!(
            wrapper_test.intersection(&partition_test).cloned().collect_vec(),
            Vec::<String>::default(),
        );
        assert_eq!(
            frequency_test.intersection(&partition_test).cloned().collect_vec(),
            Vec::<String>::default(),
        );
        Self {
            ctx,
            labels,
            frequency_test,
            wrapper_test,
            partition_test,
        }
    }
    pub fn test_roots(&self)
    {
        let label_strings = self.label_strings_set();
        let roots_test = self.ctx.get_roots_test();
        assert_eq!(
            label_strings
                .intersection(&roots_test.iter().cloned().collect())
                .into_iter()
                .cloned()
                .sorted()
                .collect_vec(),
            roots_test,
        );
    }
    pub fn test_leaves(&self)
    {
        let label_strings = self.label_strings_set();
        let leaves_test = self.ctx.get_leaves_test();
        assert_eq!(
            label_strings
                .intersection(&leaves_test.iter().cloned().collect())
                .into_iter()
                .cloned()
                .sorted()
                .collect_vec(),
            leaves_test,
        );
    }
    pub fn get_frequency_test(&self) -> Vec<String>
    {
        self.ctx
            .get_leaves_test()
            .iter()
            .chain(
                self.ctx.get_roots_test().iter()
            )
            .chain(
                self.frequency_test.iter()
            )
            .sorted()
            .cloned()
            .collect()
    }
    pub fn get_wrapper_test(&self) -> Vec<String>
    {
        self.get_frequency_test()
            .iter()
            .chain(
                self.wrapper_test.iter()
            )
            .sorted()
            .cloned()
            .collect()
    }
    pub fn get_partition_test(&self) -> Vec<String>
    {
        self.get_wrapper_test()
            .iter()
            .chain(
                self.partition_test.iter()
            )
            .sorted()
            .cloned()
            .collect()
    }
    pub fn label_strings_set(&self) -> HashSet<String> {
        self.labels
            .iter()
            .map(|vi| self.ctx.vocab.get(vi).unwrap().ngram.clone())
            .collect()
    }
    pub fn test_freq(&self)
    {
        let Self {
            ctx: TestCtx {
                vocab,
                corpus,
                leaves_test,
                roots_test,
            },
            labels,
            ..
        } = self;

        let label_strings = self.label_strings_set();
        let frequency_test = self.get_frequency_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            frequency_test,
        );
    }
    pub fn test_wrap(&self)
    {
        let Self {
            ctx: TestCtx {
                vocab,
                corpus,
                leaves_test,
                roots_test,
            },
            labels,
            ..
        } = self;
        let label_strings = self.label_strings_set();
        let wrapper_test = self.get_wrapper_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            wrapper_test,
        );
    }
    pub fn test_part(&self)
    {
        let Self {
            ctx: TestCtx {
                vocab,
                corpus,
                leaves_test,
                roots_test,
            },
            labels,
            ..
        } = self;
    }
}
pub fn test_graph()
{
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    let corpus = Corpus::new("ottos_mops".to_owned(), texts);
    // graph of all containment edges between n and n+1
    let mut image = LabellingCtx::from_corpus(&corpus);

    {
        TestCtx::new(&image.vocab, &corpus)
            .test_containment();
    }

    image.label_freq();

    {
        let ctx = LabelTestCtx::new(
            TestCtx::new(&image.vocab, &corpus),
            &image.labels,
        );
        ctx.test_roots();
        ctx.test_leaves();

        ctx.test_freq();
    }

    image.label_wrap();

    {
        let ctx = LabelTestCtx::new(
            TestCtx::new(&image.vocab, &corpus),
            &image.labels,
        );
        ctx.test_wrap();

    }

    image.label_part();

    {
        let ctx = TestCtx::new(&image.vocab, &corpus);
        let ctx = LabelTestCtx::new(ctx, &image.labels);
        ctx.test_part();
    }
}
