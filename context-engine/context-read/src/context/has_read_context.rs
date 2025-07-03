use context_trace::graph::{
    vertex::{
        child::Child,
        pattern::IntoPattern,
    },
    HypergraphRef,
};

use crate::{
    context::ReadCtx,
    sequence::ToNewTokenIndices,
};
pub trait HasReadCtx {
    fn read_context<'g>(&'g mut self) -> ReadCtx;
    fn read_sequence(&mut self) -> Option<Child> {
        self.read_context().read_sequence()
    }
    fn read_pattern(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Option<Child> {
        self.read_context().read_pattern(pattern)
    }
}

impl HasReadCtx for ReadCtx {
    fn read_context(&mut self) -> ReadCtx {
        self.clone()
    }
}
impl<T: HasReadCtx> HasReadCtx for &'_ mut T {
    fn read_context(&mut self) -> ReadCtx {
        (**self).read_context()
    }
}
impl<S: ToNewTokenIndices + Clone> HasReadCtx for (HypergraphRef, S) {
    fn read_context(&mut self) -> ReadCtx {
        let (graph, seq) = self;
        ReadCtx::new(graph.clone(), seq.clone())
    }
}
impl<S: ToNewTokenIndices + Clone> HasReadCtx for (&mut HypergraphRef, S) {
    fn read_context(&mut self) -> ReadCtx {
        let (graph, seq) = self;
        ReadCtx::new(graph.clone(), seq.clone())
    }
}
