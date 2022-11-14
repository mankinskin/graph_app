#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
#![feature(control_flow_enum)]
#![feature(pin_macro)]

extern crate test;

pub mod direction;
pub mod graph;
pub mod search;
pub mod vertex;
pub mod traversal;
pub mod index;
pub mod logger;
pub mod mock;
mod read;

pub(crate) use search::*;
pub(crate) use vertex::*;
pub(crate) use traversal::*;
pub(crate) use logger::*;
pub(crate) use index::*;

#[cfg(test)]
pub use graph::tests::*;

pub use {
    graph::{
        HypergraphRef,
        Hypergraph,
    },
    vertex::{
        Token,
        VertexKey,
        VertexData,
        Tokenize,
        Child,
        PatternId,
    },
};
#[allow(unused)]
pub(crate) use {
    tracing::*,
    tracing_test::traced_test,
    itertools::*,
    async_std::{
        sync::{
            RwLockReadGuard,
            RwLockWriteGuard,
        },
    },
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
        pin::{Pin, pin},
    },
    lazy_static::lazy_static,
    function_name::named,
    tap::{
        Tap,
        Pipe,
    },
    valuable::*,
    async_trait::async_trait,
    async_recursion::async_recursion,
    async_std::stream::{
        //Stream,
        //StreamExt,
    },
    futures::{
        task::Poll,
        stream::{
            Stream,
            StreamExt,
        },
        future::{
            OptionFuture,
            Future,
            FutureExt,
        },
    },
};
pub(crate) type DeterministicHashSet<T> =
    HashSet<T,
        BuildHasherDefault<DefaultHasher>
    >;
pub(crate) type DeterministicHashMap<K, V> =
    HashMap<K, V, BuildHasherDefault<DefaultHasher>>;