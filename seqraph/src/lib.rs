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

pub mod direction;
pub mod graph;
pub mod search;
pub mod vertex;
pub mod traversal;
pub mod index;
//pub mod logger;
pub mod mock;
pub mod split;
//pub mod read;
pub mod tests;
pub mod join;
pub mod reexports;
pub mod shared;

pub use reexports::*;
