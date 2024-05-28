pub use {
    derive_more::*,
    derive_new::*,
    itertools::*,
    maplit::hashmap,
    ngram::*,
    plotters::*,
    range_ext::intersect::Intersect,
    std::{
        borrow::Borrow,
        collections::VecDeque,
        default::Default,
        fmt::Debug,
        hash::Hash,
        ops::Range,
        path::Path,
    },
    tap::prelude::*,
};

#[cfg(not(any(test, feature = "test-hashing")))]
pub use std::collections::{
    HashMap,
    HashSet,
};
#[cfg(any(test, feature = "test-hashing"))]
use std::hash::{
    BuildHasherDefault,
    DefaultHasher,
};
#[cfg(any(test, feature = "test-hashing"))]
pub type HashSet<T> = std::collections::HashSet<T, BuildHasherDefault<DefaultHasher>>;
#[cfg(any(test, feature = "test-hashing"))]
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
