#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
#![feature(generic_associated_types)]

extern crate test;

mod direction;
mod graph;
mod search;
mod vertex;
mod traversal;
mod index;
mod read;
mod logger;

pub use direction::*;
#[cfg(test)]
pub use graph::tests::*;
pub use graph::*;
pub use search::*;
pub use vertex::*;
pub(crate) use traversal::*;
pub use read::*;
pub use logger::*;

#[allow(unused)]
pub(crate) use {
    tracing::*,
    itertools::*,
    std::fmt::Debug,
    std::ops::{
        Deref,
        DerefMut,
    },
    std::borrow::{
        Borrow,
        BorrowMut,
    },
};