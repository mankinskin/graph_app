use crate::shared::*;

pub mod vocabulary;
pub mod containment;
pub mod labelling;

pub use {
    vocabulary::*,
    labelling::*,
    containment::*,
};
use pretty_assertions::assert_eq;

//pub type HashSet<T> = std::collections::HashSet<T,
//    std::hash::BuildHasherDefault<std::hash::DefaultHasher>,
//>;

#[derive(new)]
struct TestCtx<'a> {
    vocab: Vocabulary,
    texts: Vec<String>,
    corpus: &'a [&'a str],
    leaves_test: HashSet<String>,
    roots_test: HashSet<String>,
    labels: Option<&'a HashSet<usize>>,
}
fn test_containment(ctx: &TestCtx<'_>) {
    let TestCtx {
        vocab,
        texts,
        corpus,
        leaves_test,
        roots_test,
        ..
    } = ctx;
    assert_eq!(
        vocab.leaves.iter().map(|vi|
            vocab.get(vi).unwrap().ngram.clone()
        ).collect::<HashSet<_>>(),
        *leaves_test,
    );
    assert_eq!(
        vocab.roots.iter().map(|vi|
            vocab.get(vi).unwrap().ngram.clone()
        ).collect::<HashSet<_>>(),
        *roots_test,
    );
}
fn test_labels(ctx: &TestCtx<'_>) {
    let TestCtx {
        vocab,
        texts,
        corpus,
        labels,
        leaves_test,
        roots_test,
    } = ctx;
    let (
        labels,
     ) = (
        labels.unwrap(),
     );
    let label_strings: HashSet<_> = labels.iter().map(|vi|
        vocab.get(vi).unwrap().ngram.clone()
    )
    .collect();
    println!(
        "{:#?}",
        label_strings.iter().sorted_by_key(|s| s.len()).collect_vec()
    );
    for x in &vocab.roots {
        assert!(labels.contains(x));
    }
    for x in &vocab.leaves {
        assert!(labels.contains(x));
    }
    // frequent nodes:
    // - occur in two different contexts
    // i.e. there exist two reachable parent nodes
    let frequency_test: HashSet<_> =
        leaves_test.iter().cloned()
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
            .map(ToString::to_string)
        )
        .collect();

    assert_eq!(
        label_strings.into_iter().sorted().collect_vec(),
        frequency_test.into_iter().sorted().collect_vec(),
    );
}
pub fn test_graph() {
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    
    // graph of all containment edges between n and n+1
    let vocab = containment_graph(&texts);

    let roots_test: HashSet<_> = corpus.iter().map(ToString::to_string).collect();
    let leaves_test: HashSet<_> = 
        texts.iter().flat_map(|s|
            s.chars().ngrams(1).map(String::from_iter).collect_vec()
        )
        .collect();
    let mut ctx = TestCtx::new(vocab, texts, &corpus, leaves_test, roots_test, None);
    test_containment(&ctx);

    let labels = label_vocab(&ctx.vocab);
    ctx.labels = Some(&labels);
    test_labels(&ctx);

}
