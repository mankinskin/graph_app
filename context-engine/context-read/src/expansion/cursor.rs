use context_trace::*;
use derive_more::{
    Deref,
    DerefMut,
};
use derive_new::new;

use crate::context::ReadCtx;

#[derive(Debug, Deref, DerefMut, new)]
pub struct CursorCtx<'a> {
    #[deref]
    #[deref_mut]
    pub ctx: ReadCtx,
    pub cursor: &'a mut PatternRangePath,
}
