#![allow(non_snake_case, unused)]
#![feature(hash_extract_if)]

pub mod count;
pub mod graph;
pub mod shared;
use shared::*;
pub use {
    count::*,
    graph::*,
};

const OTTOS_MOPS_CORPUS: [&'static str; 4] = [
    "ottos mops trotzt",
    "otto: fort mops fort",
    "ottos mops hopst fort",
    "otto: soso",
];

fn main() {
    //test_ngrams()
    graph::test_graph()
}

pub fn test_ngrams() {}
fn read_corpus(file_path: impl AsRef<Path>) -> String {
    //let corpus: String = String::from("fldfjdlsjflskdjflsdfaädüwwrivfokl");
    let mut csv = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(file_path)
        .expect("Corpus file not found.");
    csv.records()
        .into_iter()
        .map(|r| r.unwrap()[1].to_string())
        .join(" ")
}
