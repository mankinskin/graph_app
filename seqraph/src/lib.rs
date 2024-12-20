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

#[macro_use]
extern crate hypercontext_api;

extern crate test;

pub mod direction;
pub mod insert;
pub mod search;
//pub mod logger;
pub mod mock;
pub mod split;
pub mod read;
pub mod join;
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
