use super::TraceCtx;
use crate::trace::HasGraph;

pub trait Traceable {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    );
}
