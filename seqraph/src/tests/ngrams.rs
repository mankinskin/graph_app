use ngram::*;
use crate::*;
use std::{
    path::Path,
    fs::File,
};

fn read_corpus(file_path: impl AsRef<Path>) -> String {
    //let corpus: String = String::from("fldfjdlsjflskdjflsdfaädüwwrivfokl");
    let mut csv = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(file_path).expect("Corpus file not found.");
    csv.records().into_iter().map(|r| r.unwrap()[1].to_string()).join(" ")
}
//#[test]
pub fn test_ngrams() {
    let file_path = "./corpus/eng_news_2020_100K/eng_news_2020_100K-sentences.txt";
    let corpus: String = read_corpus(file_path);

    println!("Finished reading {}", file_path);
    let N: usize = corpus.len();
    let N_MAX: usize = 10;
    let sets: Vec<HashMap<String, usize>> = (1..N_MAX).into_iter().map(|n| {
        let ngrams = corpus.chars().ngrams(n);
        ngrams.fold((0, HashMap::default()), |(mut len, mut set), x| {
            let x = String::from_iter(x);
            //println!("{}", x);
            set.entry(x)
                .and_modify(|v| *v += 1)
                .or_insert_with(|| {
                    len += 1;
                    1 as usize
                });
            (len, set)
        }).tap(|(len, _)|
            println!("Finished counting n={}: {}", n, len)
        ).1
    }).collect();
    //const K: usize = ;
    println!("All n counted");
    let C_MAX: usize = 20;
    let hist = sets.into_iter().enumerate().fold(vec![0; C_MAX], |mut hist, (_, set)| {
        for (c, h) in hist.iter_mut().enumerate() {
            *h += set.iter().filter(|(_, &v)| v == c + 1).count()
        }
        hist
    });
    for (c, n) in hist.iter().enumerate() {
        println!("{}: {}", c + 1, n);
    }
}