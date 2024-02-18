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
type HashSet<T> = std::collections::HashSet<T,
    std::hash::BuildHasherDefault<std::hash::DefaultHasher>,
>;


pub fn test_graph() {
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    let mut vocab = containment_graph(&texts);
    //println!("{:#?}", vocab.roots);
    //println!("{:#?}", vocab.leaves);
    //let x = crate::OTTOS_MOPS_CORPUS[0];
    //println!("{:#?}", vocab.get(&String::from("otto")).unwrap().data.parents);

    let leaves_test: HashSet<_> = 
        texts.iter().flat_map(|s|
            s.chars().ngrams(1).map(String::from_iter).collect_vec()
        )
        .collect();
    assert_eq!(
        vocab.leaves.iter().map(|vi|
            vocab.get(vi).unwrap().ngram.clone()
        ).collect::<HashSet<_>>(),
        leaves_test,
    );
    let roots_test: HashSet<_> = corpus.iter().map(ToString::to_string).collect();
    assert_eq!(
        vocab.roots.iter().map(|vi|
            vocab.get(vi).unwrap().ngram.clone()
        ).collect::<HashSet<_>>(),
        roots_test,
    );
    let labels = label_vocab(&vocab);
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
        .chain(roots_test.into_iter())
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

    //.chain(
    //    
    //)
    //.collect();
    //println!("{:?}", roots);
    //println!("{}", vocab.len());
    //println!("{:?}", bft.stats);
}
