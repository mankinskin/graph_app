use derive_more::derive::From;

use crate::{
    graph::vertex::{
        location::{
            child::ChildLocation,
            pattern::{
                IntoPatternLocation,
                PatternLocation,
            },
        },
        pattern::Pattern,
    },
    path::accessors::root::RootPattern,
};

#[derive(Clone, Debug, PartialEq, Eq, From)]
pub struct IndexRoot {
    pub location: PatternLocation,
}
pub trait PathRoot: Clone + RootPattern {}

impl PathRoot for Pattern {}

impl PathRoot for IndexRoot {}

pub trait RootedPath {
    type Root: PathRoot;
    fn path_root(&self) -> Self::Root;
}
impl RootedPath for ChildLocation {
    type Root = IndexRoot;
    fn path_root(&self) -> Self::Root {
        IndexRoot::from(self.into_pattern_location())
    }
}
