pub mod chain;
pub mod cursor;
pub mod link;
pub mod stack;

use crate::{
    complement::ComplementBuilder,
    context::ReadCtx,
    expansion::{
        chain::{
            band::Band,
            expand::ExpandCtx,
            link::{
                BandCap,
                BandExpansion,
                ChainOp,
            },
        },
        cursor::CursorCtx,
        link::ExpansionLink,
    },
};
use chain::BandChain;

use context_insert::insert::ToInsertCtx;
use context_trace::{
    graph::{
        getters::{
            ErrorReason,
            IndexWithPath,
        },
        vertex::child::Child,
    },
    path::{
        accessors::role::End,
        structs::{
            query_range_path::FoldablePath,
            rooted::pattern_range::PatternRangePath,
        },
        RolePathUtils,
    },
    trace::child::bands::HasChildRoleIters,
};

use derive_more::{
    Deref,
    DerefMut,
};
use tracing::debug;

#[derive(Debug, Deref, DerefMut)]
pub struct ExpansionCtx<'a> {
    #[deref]
    #[deref_mut]
    cursor: CursorCtx<'a>,
    chain: BandChain,
}
impl Iterator for ExpansionCtx<'_> {
    type Item = Child;

    fn next(&mut self) -> Option<Self::Item> {
        ExpandCtx::try_new(self)
            .and_then(|mut ctx| {
                ctx.find_map(|op| match &op {
                    ChainOp::Expansion(_) => Some(op),
                    ChainOp::Cap(cap) =>
                        self.chain.ends_at(cap.start_bound).map(|_| op),
                })
            })
            .and_then(|op| match op {
                ChainOp::Expansion(exp) => Some(self.apply_expansion(exp)),
                ChainOp::Cap(cap) => self.apply_cap(cap),
            })
    }
}
impl<'a> ExpansionCtx<'a> {
    pub fn new(
        trav: ReadCtx,
        cursor: &'a mut PatternRangePath,
    ) -> Self {
        debug!("New ExpansionCtx");
        let IndexWithPath { index: first, path } =
            match trav.insert_or_get_complete(cursor.clone()) {
                Ok(Ok(root)) => root,
                Ok(Err(root)) => root,
                Err(ErrorReason::SingleIndex(c)) => *c,
                Err(_) => IndexWithPath {
                    index: cursor.start_index(&trav),
                    path: cursor.clone(),
                },
            };
        *cursor = path;

        Self {
            chain: BandChain::new(first),
            cursor: CursorCtx::new(trav, cursor),
        }
    }
    pub fn last(&self) -> &Band {
        self.chain.last().unwrap().band
    }
    pub fn find_largest_bundle(self) -> <Self as Iterator>::Item {
        debug!("find_largest_bundle");
        let first = self.chain.first().unwrap().postfix();
        self.last().unwrap_or(first)
    }
    pub fn cursor_root_index(&self) -> &Child {
        self.chain
            .bands
            .first()
            .expect("no overlaps in chain")
            .pattern
            .last()
            .expect("empty pattern")
    }
    pub fn apply_expansion(
        &mut self,
        exp: BandExpansion,
    ) -> <Self as Iterator>::Item {
        debug!("apply_expansion");
        *self.cursor.cursor = exp.expansion.path.clone();

        // handle case where expansion can be inserted after stack head (first band in current stack)
        let link = self.create_expansion_link(&exp);
        let complement =
            ComplementBuilder::new(link).build(&mut self.cursor.ctx);
        // TODO: Change this to a stack (list of overlaps with back contexts)
        self.chain
            .append_front_complement(complement, exp.expansion.index);

        exp.expansion.index
    }
    pub fn apply_cap(
        &mut self,
        cap: BandCap,
    ) -> Option<<Self as Iterator>::Item> {
        debug!("apply_cap");
        let mut first = self.chain.bands.pop_first().unwrap();
        first.append(cap.expansion);
        self.chain.append(first);
        None
    }
    fn create_expansion_link(
        &self,
        exp: &BandExpansion,
    ) -> ExpansionLink {
        debug!("create_expansion_link");
        let BandExpansion {
            postfix_path,
            expansion:
                IndexWithPath {
                    index: expansion, ..
                },
            start_bound,
        } = exp;
        let start_bound = *start_bound;
        let overlap = postfix_path.role_leaf_child::<End, _>(&self.cursor.ctx);
        let prefix_path = expansion.prefix_path(&self.cursor.ctx, overlap);

        ExpansionLink {
            start_bound,
            root_postfix: postfix_path.clone(),
            expansion_prefix: prefix_path,
        }
    }
}
