use std::fmt::Debug;

use crate::{
    graph::vertex::location::SubLocation,
    traversal::cache::entry::position::{
        Offset,
        SubSplitLocation,
    },
    HashMap,
};

pub type OffsetLocations = HashMap<Offset, Vec<SubSplitLocation>>;
pub type CompleteLocations = HashMap<Offset, Result<Vec<SubSplitLocation>, SubLocation>>;

pub trait NodeSplitOutput<S>: Default {
    fn set_root_mode(
        &mut self,
        _root_mode: RootMode,
    ) {
    }
    fn splits_mut(&mut self) -> &mut S;
}

impl NodeSplitOutput<Self> for OffsetLocations {
    fn splits_mut(&mut self) -> &mut OffsetLocations {
        self
    }
}

impl NodeSplitOutput<OffsetLocations> for (OffsetLocations, RootMode) {
    fn set_root_mode(
        &mut self,
        root_mode: RootMode,
    ) {
        self.1 = root_mode;
    }
    fn splits_mut(&mut self) -> &mut OffsetLocations {
        &mut self.0
    }
}

pub trait NodeType {
    type GlobalSplitOutput: NodeSplitOutput<OffsetLocations>;
    type CompleteSplitOutput: Default;
    fn map(
        global: Self::GlobalSplitOutput,
        f: impl Fn(OffsetLocations) -> CompleteLocations,
    ) -> Self::CompleteSplitOutput;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RootMode {
    Prefix,
    Postfix,
    Infix,
}

impl Default for RootMode {
    fn default() -> Self {
        Self::Infix
    }
}

pub struct RootNode;

impl NodeType for RootNode {
    type GlobalSplitOutput = (OffsetLocations, RootMode);
    type CompleteSplitOutput = (CompleteLocations, RootMode);
    fn map(
        global: Self::GlobalSplitOutput,
        f: impl Fn(OffsetLocations) -> CompleteLocations,
    ) -> Self::CompleteSplitOutput {
        (f(global.0), global.1)
    }
}

pub struct InnerNode;

impl NodeType for InnerNode {
    type GlobalSplitOutput = OffsetLocations;
    type CompleteSplitOutput = CompleteLocations;
    fn map(
        global: Self::GlobalSplitOutput,
        f: impl Fn(OffsetLocations) -> CompleteLocations,
    ) -> Self::CompleteSplitOutput {
        f(global)
    }
}
