//pub mod bundle;
pub mod chain;

use crate::{
    complement::ComplementBuilder,
    context::ReadCtx,
    expansion::chain::{
        BandCap,
        BandExpansion,
        ChainCtx,
        ChainOp,
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
            rooted::{
                pattern_range::PatternRangePath,
                role_path::{
                    IndexEndPath,
                    IndexStartPath,
                },
            },
        },
        RolePathUtils,
    },
    trace::child::bands::HasChildRoleIters,
};

use derive_more::{
    Deref,
    DerefMut,
};

#[derive(Debug)]
pub struct ExpansionLink {
    pub expansion_prefix: IndexStartPath,
    pub root_postfix: IndexEndPath,
    pub start_bound: usize,
}

#[derive(Debug, Deref, DerefMut)]
pub struct ExpansionCtx<'cursor> {
    #[deref]
    #[deref_mut]
    chain_ops: ChainCtx<'cursor>,
}
impl Iterator for ExpansionCtx<'_> {
    type Item = Child;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chain_ops.next() {
            Some(op) => match op {
                ChainOp::Expansion(exp) => Some(self.apply_expansion(exp)),
                ChainOp::Cap(cap) => self.apply_cap(cap),
            },
            None => None,
        }
    }
}
impl<'a> ExpansionCtx<'a> {
    pub fn new(
        trav: ReadCtx,
        cursor: &'a mut PatternRangePath,
    ) -> Self {
        let inner_cursor = cursor.clone();
        let IndexWithPath { index: first, path } =
            match trav.insert_or_get_complete(inner_cursor) {
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
            chain_ops: ChainCtx::new(trav, cursor, BandChain::new(first)),
            //bundle: Bundle::new(Band::from(first)),
        }
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
        *self.cursor = exp.expansion.path.clone();

        let link = self.create_expansion_link(&exp);
        let complement = ComplementBuilder::new(link).build(&mut self.trav);
        self.chain
            .append_front_complement(complement, exp.expansion.index);

        exp.expansion.index
    }
    pub fn apply_cap(
        &mut self,
        cap: BandCap,
    ) -> Option<<Self as Iterator>::Item> {
        let mut first = self.chain_ops.chain.bands.pop_first().unwrap();
        first.append(cap.expansion);
        self.chain_ops.chain.append(first);
        None
    }
    pub fn find_largest_bundle(mut self) -> <Self as Iterator>::Item {
        // so we have an iterator but only call it once?
        // should run over all expansions and update a root cache object
        // when iterator ends, read root cache
        self.next().unwrap_or_else(|| {
            self.chain_ops.chain.pop_first().unwrap().postfix()
        })
    }
    fn create_expansion_link(
        &self,
        exp: &BandExpansion,
    ) -> ExpansionLink {
        let BandExpansion {
            postfix_path,
            expansion:
                IndexWithPath {
                    index: expansion, ..
                },
            start_bound,
        } = exp;
        let start_bound = *start_bound;
        let overlap = postfix_path.role_leaf_child::<End, _>(&self.trav);
        let prefix_path = expansion.prefix_path(&self.trav, overlap);

        ExpansionLink {
            start_bound,
            root_postfix: postfix_path.clone(),
            expansion_prefix: prefix_path,
        }
    }
}
