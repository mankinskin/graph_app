use derive_more::derive::From;

use crate::graph::vertex::{
    location::pattern::PatternLocation,
    pattern::Pattern,
};

#[derive(Clone, Debug, PartialEq, Eq, From)]
pub struct IndexRoot {
    pub location: PatternLocation,
}
pub trait PathRoot {}

impl PathRoot for Pattern {}

impl PathRoot for IndexRoot {}

pub trait RootedPath {
    type Root: PathRoot;
    fn path_root(&self) -> &Self::Root;
}
