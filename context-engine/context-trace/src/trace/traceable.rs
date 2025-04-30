use super::TraceContext;
use crate::trace::HasGraph;

pub trait Traceable {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceContext<G>,
    );
}
