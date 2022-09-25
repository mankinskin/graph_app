#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
#![feature(generic_associated_types)]
#![feature(map_first_last)]

extern crate test;

mod direction;
mod graph;
mod search;
mod vertex;
mod traversal;
mod index;
mod logger;
mod mock;
//mod read;

pub use direction::*;
pub use graph::*;
pub use search::*;
pub use vertex::*;
pub(crate) use traversal::*;
pub use logger::*;
pub use index::*;
//pub use read::*;

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
        collections::{
            HashSet,
            HashMap,
            hash_map::DefaultHasher,
        },
        hash::{
            BuildHasherDefault,
            Hash,
        },
        num::NonZeroUsize,
    },
    function_name::named,
    tap::Tap,
};
pub(crate) type DeterministicHashSet<T> =
    HashSet<T,
        BuildHasherDefault<DefaultHasher>
    >;
pub(crate) type DeterministicHashMap<K, V> =
    HashMap<K, V, BuildHasherDefault<DefaultHasher>>;