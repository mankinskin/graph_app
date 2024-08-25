use itertools::Itertools;
use ngram::NGram;
use seqraph::HashSet;
use crate::graph::labelling::LabellingCtx;
use crate::graph::vocabulary::{Corpus, Vocabulary};
use crate::graph::vocabulary::entry::HasVertexEntries;
#[derive(Debug)]
pub struct TestCtx<'a>
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

    pub(crate) fn test_containment(&self)
    {
        let Self {
            vocab,
            corpus,
            leaves_test,
            roots_test,
            ..
        } = self;
        pretty_assertions::assert_eq!(
            vocab
                .leaves
                .iter()
                .map(|vi| { vocab.get_vertex(vi).unwrap().ngram.clone() })
                .collect::<HashSet<_>>(),
            *leaves_test,
        );
        pretty_assertions::assert_eq!(
            vocab
                .roots
                .iter()
                .map(|vi| { vocab.get_vertex(vi).unwrap().ngram.clone() })
                .collect::<HashSet<_>>(),
            *roots_test,
        );
    }
}
#[derive(Debug)]
pub struct LabelTestCtx<'a>
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
        let frequency_test: HashSet<String> =
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
        let wrapper_test: HashSet<String> =
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
        let partition_test: HashSet<String> =
            (&[
            ] as &[&'static str])
                .into_iter()
                .map(ToString::to_string)
                .collect();
        for (a, b) in [
            &frequency_test,
            &wrapper_test,
            &partition_test,
        ].into_iter().combinations(2)
            .map(|v| (v[0], v[1]))
        {
            assert_eq!(
                a.intersection(&b).cloned().collect_vec(),
                Vec::<String>::default(),
            );
        }
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
            .map(|vi| self.ctx.vocab.get_vertex(vi).unwrap().ngram.clone())
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
        let label_strings = self.label_strings_set();
        let wrapper_test = self.get_partition_test();

        assert_eq!(
            label_strings.iter().cloned().sorted().collect_vec(),
            wrapper_test,
        );
    }
}
