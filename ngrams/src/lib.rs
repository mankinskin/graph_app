#![allow(non_snake_case, unused)]
#![feature(hash_extract_if)]

use std::path::Path;

use itertools::Itertools;

#[cfg(not(debug_assertions))]
pub use {
    count::*,
    graph::*,
    shared::*,
};

pub mod graph;
#[cfg(not(debug_assertions))]
mod shared;
pub mod tests;