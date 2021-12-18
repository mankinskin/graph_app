#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]

extern crate test;

mod graph;
mod r#match;
mod search;
mod read;
mod merge;
mod split;
mod vertex;

#[cfg(test)]
pub use graph::tests::*;
pub use graph::*;
pub(crate) use merge::*;
pub use split::*;
pub(crate) use read::*;
pub use r#match::*;
pub use search::*;
pub use vertex::*;
