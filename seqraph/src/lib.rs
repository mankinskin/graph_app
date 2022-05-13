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
mod gen_graph;

pub use direction::*;
#[cfg(test)]
pub use graph::tests::*;
pub use graph::*;
pub use search::*;
pub use vertex::*;
pub(crate) use traversal::*;
pub use read::*;
pub use logger::*;
pub use gen_graph::*;

#[allow(unused)]
pub(crate) use {
    tracing::*,
    itertools::*,
    std::{
        fmt::Debug,
        ops::{
            Deref,
            DerefMut,
            ControlFlow,
        },
        borrow::{
            Borrow,
            BorrowMut,
        },
    },
};

#[cfg(test)]
mod tests {
    #[test]
    fn fuzz1() {
        if crate::gen_graph::gen_graph().is_err() {
            panic!();
        }
    }
}