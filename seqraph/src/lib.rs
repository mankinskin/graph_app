#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]
#![feature(try_blocks)]
#![feature(hash_drain_filter)]

extern crate test;

mod direction;
mod graph;
mod r#match;
mod search;
//mod read;
//mod merge;
mod vertex;
//mod split;
//mod index;

pub use direction::*;
#[cfg(test)]
pub use graph::tests::*;
pub use graph::*;
//pub use split::*;
//pub(crate) use read::*;
pub use r#match::*;
pub use search::*;
pub use vertex::*;
