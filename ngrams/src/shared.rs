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

pub use {
    itertools::*,
    ngram::*,
    range_ext::intersect::Intersect,
    std::{
        borrow::Borrow,
        default::Default,
        fmt::Debug,
        hash::Hash,
    },
    tap::prelude::*,
};

#[cfg(any(test, feature = "test-hashing"))]
pub type HashSet<T> =
    std::collections::HashSet<T, BuildHasherDefault<DefaultHasher>>;

#[cfg(any(test, feature = "test-hashing"))]
pub type HashMap<K, V> =
    std::collections::HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
