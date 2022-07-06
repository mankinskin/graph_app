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
mod mock;

pub use direction::*;
pub use graph::*;
pub use search::*;
pub use vertex::*;
pub(crate) use traversal::*;
pub use read::*;
pub use logger::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use graph::tests::*;

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
            Range,
            RangeInclusive,
        },
        cmp::Ordering,
        borrow::{
            Borrow,
            BorrowMut,
        },
        marker::PhantomData,
        sync::{
            RwLockReadGuard,
            RwLockWriteGuard,
        },
    },
    function_name::named,
};