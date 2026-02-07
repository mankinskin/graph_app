#![allow(non_snake_case, unused)]
#![feature(mapped_lock_guards)]

use std::path::Path;

use itertools::Itertools;

#[cfg(not(debug_assertions))]
pub(crate) use {
    graph::*,
    shared::*,
};

pub mod cancellation;
pub mod graph;
#[cfg(not(debug_assertions))]
mod shared;
pub(crate) mod storage;
pub(crate) mod tests;

pub use crate::{
    cancellation::Cancellation,
    graph::{
        Status,
        vocabulary::ProcessStatus,
    },
};
