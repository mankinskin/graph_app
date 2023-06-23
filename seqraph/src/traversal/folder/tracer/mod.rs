use crate::*;

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: SplitKey,
}
