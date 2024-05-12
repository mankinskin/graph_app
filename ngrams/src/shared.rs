pub use {
    ngram::*,
    maplit::hashmap,
    tap::prelude::*,
    itertools::*,
    plotters::*,
    seqraph::*,
    derive_more::*,
    derive_new::*,
    seqraph::*,
    std::{
        default::Default,
        borrow::Borrow,
        fmt::Debug,
        ops::Range,
        collections::VecDeque,
        path::Path,
        hash::Hash,
    },
    range_ext::intersect::Intersect,
};

#[cfg(not(any(test, feature = "test-hashing")))]
pub use std::collections::{
    HashSet,
    HashMap,
};
#[cfg(any(test, feature = "test-hashing"))]
use std::hash::{
    BuildHasherDefault,
    DefaultHasher,
};
#[cfg(any(test, feature = "test-hashing"))]
pub type HashSet<T> =
    std::collections::HashSet<
        T,
        BuildHasherDefault<DefaultHasher>
    >;
#[cfg(any(test, feature = "test-hashing"))]
pub type HashMap<K, V> =
    std::collections::HashMap<K, V, BuildHasherDefault<DefaultHasher>>;