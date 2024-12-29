#![deny(clippy::disallowed_methods)]
#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
//#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
//#![feature(pin_macro)]
#![feature(exact_size_is_empty)]
#![feature(associated_type_defaults)]
//#![feature(return_position_impl_trait_in_trait)]

extern crate test;

pub mod graph;
pub mod direction;
pub mod traversal;
pub mod path;
pub mod mock;

pub mod split;
pub mod partition;
pub mod join;

#[cfg(test)]
pub mod tests;

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
