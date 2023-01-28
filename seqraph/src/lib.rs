#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
#![feature(control_flow_enum)]
#![feature(pin_macro)]
#![feature(exact_size_is_empty)]
#![feature(associated_type_defaults)]

extern crate test;

pub mod direction;
pub mod graph;
pub mod search;
pub mod vertex;
pub mod traversal;
//pub mod index;
pub mod logger;
pub mod mock;
//pub mod read;

pub use search::*;
pub use vertex::*;
pub use traversal::*;
pub use logger::*;
pub use direction::*;
//pub use index::*;
//pub use read::*;

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
pub use {
    tracing::*,
    tracing_test::traced_test,
    itertools::*,
    std::{
        fmt::Debug,
        ops::{
            Deref,
            DerefMut,
            ControlFlow,
            Range,
            RangeInclusive,
            RangeFrom,
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
        sync::{
            Arc,
            RwLock,
            RwLockReadGuard,
            RwLockWriteGuard,
        },
    },
    //tracing_mutex::{
    //    stdsync::{
    //        TracingRwLock as RwLock,
    //        TracingReadGuard as RwLockReadGuard,
    //        TracingWriteGuard as RwLockWriteGuard,
    //    },
    //},
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
pub type DeterministicHashSet<T> =
    HashSet<T,
        BuildHasherDefault<DefaultHasher>
    >;
pub type DeterministicHashMap<K, V> =
    HashMap<K, V, BuildHasherDefault<DefaultHasher>>;