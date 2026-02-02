#![allow(non_snake_case, unused)]
#![feature(mapped_lock_guards)]

use std::path::Path;

use itertools::Itertools;

#[cfg(not(debug_assertions))]
pub use {
    graph::*,
    shared::*,
};

pub mod cancellation;
pub mod graph;
#[cfg(not(debug_assertions))]
mod shared;
pub mod storage;
pub mod tests;
