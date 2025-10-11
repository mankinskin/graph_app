use crate::trace::{
    HasGraph,
    TraceCtx,
};

pub trait Traceable {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    );
}
