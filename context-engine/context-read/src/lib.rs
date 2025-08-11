#![deny(clippy::disallowed_methods)]
#![feature(test)]
#![feature(assert_matches)]
#![feature(try_blocks)]
//#![feature(hash_drain_filter)]
#![feature(slice_pattern)]
//#![feature(pin_macro)]
#![feature(exact_size_is_empty)]
#![feature(associated_type_defaults)]
#![feature(vec_pop_if)]
//#![feature(return_position_impl_trait_in_trait)]

extern crate test;

//pub mod bundle;
pub mod complement;
pub mod context;
pub mod expansion;
//pub mod overlap;
pub mod sequence;
//#[cfg(test)]
//mod tests;

#[cfg(test)]
mod tests;
