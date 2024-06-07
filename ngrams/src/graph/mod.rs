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
    corpus: Corpus,
    roots_test: HashSet<String>,
    leaves_test: HashSet<String>,
    //labels: Option<&'a HashSet<usize>>,
}
impl<'a> TestCtx<'a>
{
    pub fn new(
        vocab: &'a Vocabulary,
        corpus: Corpus,
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
#[derive(Debug, new)]
struct LabelTestCtx<'a>
{
    ctx: TestCtx<'a>,
    labels: HashSet<usize>,
}
impl<'a> LabelTestCtx<'a> {
    pub fn test_labels(&self)
    {
        let Self {
            ctx: TestCtx {
                vocab,
                corpus,
                leaves_test,
                roots_test,
            },
            labels,
        } = self;
        let label_strings: HashSet<_> = labels
            .iter()
            .map(|vi| vocab.get(vi).unwrap().ngram.clone())
            .collect();
        println!(
            "{:#?}",
            label_strings
                .iter()
                .sorted_by_key(|s| s.len())
                .collect_vec()
        );
        for x in &vocab.roots
        {
            assert!(labels.contains(x));
        }
        for x in &vocab.leaves
        {
            assert!(labels.contains(x));
        }
        // frequent nodes:
        // - occur in two different contexts
        // i.e. there exist two reachable parent nodes
        let frequency_test: HashSet<_> = leaves_test
            .iter()
            .cloned()
            .chain(roots_test.into_iter().cloned())
            .chain(
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
                    .map(ToString::to_string),
            )
            .collect();

        assert_eq!(
        label_strings.into_iter().sorted().collect_vec(),
        frequency_test.into_iter().sorted().collect_vec(),
    );
    }
}
pub fn test_graph()
{
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    let corpus = Corpus::new("ottos_mops".to_owned(), texts);
    // graph of all containment edges between n and n+1
    let mut image = LabellingCtx::from_corpus(&corpus);

    let ctx = TestCtx::new(&image.vocab, corpus);
    ctx.test_containment();
    let corpus = ctx.corpus;
    let labels = image
        .label_vocab();
    LabelTestCtx::new(TestCtx::new(&image.vocab, corpus), labels)
        .test_labels();
}
