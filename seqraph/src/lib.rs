#![deny(clippy::disallowed_methods)]

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
//#![feature(return_position_impl_trait_in_trait)]

extern crate test;

pub mod direction;
pub mod graph;
pub mod search;
pub mod vertex;
pub mod traversal;
pub mod index;
pub mod logger;
pub mod mock;
pub mod split;
//pub mod read;
pub mod tests;


#[cfg(test)]
pub use graph::tests::*;
#[allow(unused)]
pub use {
    petgraph::graph::DiGraph,
    auto_impl::auto_impl,
    tracing::*,
    linked_hash_set::*,
    linked_hash_map::*,
    tracing_test::traced_test,
    itertools::*,
    derive_more::{
        Add,
        Sub,
        Deref,
        DerefMut,
        IntoIterator
    },
    derive_new::*,
    std::{
        fmt::{
            Debug,
            Display,
        },
        ops::{
            Deref,
            DerefMut,
            ControlFlow,
            Range,
            RangeInclusive,
            RangeFrom,
            RangeTo,
        },
        slice::SliceIndex,
        convert::TryInto,
        cmp::Ordering,
        borrow::{
            Borrow,
            BorrowMut,
        },
        marker::PhantomData,
        collections::{
            hash_map::DefaultHasher,
            VecDeque,
            BTreeMap,
            BTreeSet,
            binary_heap::BinaryHeap,
        },
        hash::{
            BuildHasherDefault,
            Hash,
            Hasher,
        },
        num::NonZeroUsize,
        pin::{Pin, pin},
        sync::{
            Arc,
            RwLock,
            RwLockReadGuard,
            RwLockWriteGuard,
            Mutex,
            atomic::{
                self,
                AtomicUsize,
            },
        },
        iter::{
            FromIterator,
            Extend,
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
    derive_more::*,
};
pub use {
    search::*,
    vertex::*,
    traversal::*,
    logger::*,
    direction::*,
    index::*,
    //read::*,
    split::*,
    graph::*,
    vertex::{
        Token,
        VertexKey,
        VertexData,
        Tokenize,
        Child,
        PatternId,
    },
};
pub type HashSet<T> =
    std::collections::HashSet<T,
        BuildHasherDefault<DefaultHasher>
    >;
pub type HashMap<K, V> =
    std::collections::HashMap<K, V, BuildHasherDefault<DefaultHasher>>;