use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;

use seqraph::HashSet;

use crate::graph::vocabulary::Corpus;
use crate::graph::{
    labelling::label_vocab,
    vocabulary::{
        IndexVocab,
        Vocabulary,
    },
};

mod containment;
mod labelling;
mod partitions;
mod traversal;
mod vocabulary;

#[derive(Debug)]
struct TestCtx<'a>
{
    vocab: Vocabulary,
    corpus: Corpus,
    roots_test: HashSet<String>,
    leaves_test: HashSet<String>,
    labels: Option<&'a HashSet<usize>>,
}
impl<'a> TestCtx<'a>
{
    pub fn new(
        vocab: Vocabulary,
        corpus: Corpus,
        labels: Option<&'a HashSet<usize>>,
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
            labels,
        }
    }
}
fn test_containment(ctx: &TestCtx<'_>)
{
    let TestCtx {
        vocab,
        corpus,
        leaves_test,
        roots_test,
        ..
    } = ctx;
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
fn test_labels(ctx: &TestCtx<'_>)
{
    let TestCtx {
        vocab,
        corpus,
        labels,
        leaves_test,
        roots_test,
    } = ctx;
    let labels = labels.unwrap();
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
pub fn test_graph()
{
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    let corpus = Corpus::new("ottos_mops".to_owned(), texts);
    // graph of all containment edges between n and n+1
    let vocab = Vocabulary::from_corpus(&corpus);
    let mut ctx = TestCtx::new(vocab, corpus, None);

    test_containment(&ctx);

    let labels = label_vocab(&mut ctx.vocab);
    ctx.labels = Some(&labels);
    test_labels(&ctx);
}
