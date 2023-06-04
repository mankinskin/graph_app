use crate::*;

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: SplitKey,
}

//pub struct Tracer<V: NodeVisitor> {
//    frontier: _,
//    iterator: V,
//}
//
//impl Tracer {
//    pub fn new() -> Self {
//
//    }
//}