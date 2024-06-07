pub mod new;

use new::*;
use std::num::NonZeroUsize;

pub mod vertex;

pub mod position;

use crate::{
    traversal::folder::state::RootMode,
    vertex::location::SubLocation,
    HashMap,
};
use position::*;

pub type StateDepth = usize;
pub type Offset = NonZeroUsize;
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
