#![allow(non_snake_case, unused)]
#![feature(hash_extract_if)]
#![feature(iter_repeat_n)]

use std::path::Path;

use itertools::Itertools;

#[cfg(not(debug_assertions))]
pub use {
    count::*,
    graph::*,
    shared::*,
};

mod count;
mod graph;
#[cfg(not(debug_assertions))]
mod shared;

const OTTOS_MOPS_CORPUS: [&'static str; 4] = [
    "ottos mops trotzt",
    "otto: fort mops fort",
    "ottos mops hopst fort",
    "otto: soso",
];

fn main()
{
    graph::test_graph()
}

fn read_corpus(file_path: impl AsRef<Path>) -> String
{
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
