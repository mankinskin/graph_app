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

extern crate hypercontext_api;

extern crate test;

pub mod direction;
//pub mod search;
pub mod join;
//pub mod insert;
//pub mod read;

//#[cfg(test)]
//pub mod tests;
