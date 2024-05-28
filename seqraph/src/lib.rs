#![deny(clippy::disallowed_methods)]
#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
//#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
#![feature(control_flow_enum)]
//#![feature(pin_macro)]
#![feature(exact_size_is_empty)]
#![feature(associated_type_defaults)]
//#![feature(return_position_impl_trait_in_trait)]

extern crate test;

#[cfg(not(any(test, feature = "test-hashing")))]
use std::collections::{
    HashMap,
    HashSet,
};
#[cfg(any(test, feature = "test-hashing"))]
use std::hash::{
    BuildHasherDefault,
    DefaultHasher,
};

//use reexports::*;

pub mod direction;
pub mod graph;
pub mod index;
pub mod search;
pub mod traversal;
pub mod vertex;
//pub mod logger;
pub mod mock;
pub mod split;
//pub mod read;
pub mod join;
pub mod tests;
//pub mod reexports;

#[cfg(any(test, feature = "test-hashing"))]
pub type HashSet<T> = std::collections::HashSet<T, BuildHasherDefault<DefaultHasher>>;
#[cfg(any(test, feature = "test-hashing"))]
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
