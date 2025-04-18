use super::TraceContext;
use crate::trace::Traversable;

pub trait Traceable {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<Trav>,
    );
}
