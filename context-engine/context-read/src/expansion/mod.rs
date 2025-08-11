//pub mod bundle;
pub mod chain;

use crate::{
    complement::ComplementBuilder,
    context::ReadCtx,
    expansion::chain::{
        expand::ExpandCtx,
        BandCap,
        BandExpansion,
        ChainOp,
    },
};
use chain::{
    band::Band,
    BandChain,
};

//use bundle::Bundle;
use context_insert::insert::{
    result::IndexWithPath,
    ToInsertCtx,
};
use context_trace::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            wide::Wide,
        },
    },
    path::{
        accessors::{
            child::root::PatternRootChild,
            has_path::HasRolePath,
            role::{
                End,
                Start,
            },
        },
        mutators::append::PathAppend,
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::{
                    IndexEndPath,
                    IndexStartPath,
                    RootedRolePath,
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
use itertools::{
    FoldWhile,
    Itertools,
};

#[derive(Debug)]
pub struct ExpansionLink {
    pub expansion_prefix: IndexStartPath,
    pub root_postfix: IndexEndPath,
    pub start_bound: usize,
}

// # BlockIter
// | item: new indexed block
// |--# Expansions
// |  | item: Expanded/Capped index with location path in current root
// |  |--# Postfixes
// |  |  | item: Location Path to Postfix in last expansion
// |  |  |
// |  |  |

use context_trace::{
    self,
};
use derive_new::new;

#[derive(Debug, new)]
pub struct ChainCtx<'a> {
    pub trav: ReadCtx,
    pub cursor: &'a mut PatternRangePath,
    pub chain: BandChain,
}
impl<'a> ChainCtx<'a> {
    pub fn last(&self) -> &Band {
        &self.chain.last().unwrap().band
    }
}
impl<'a> Iterator for ChainCtx<'a> {
    type Item = ChainOp;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut ctx) = ExpandCtx::try_new(self) {
            ctx.find_map(|op| match &op {
                ChainOp::Expansion(_) => Some(op),
                ChainOp::Cap(cap) =>
                    self.chain.ends_at(cap.start_bound).map(|_| op),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct ExpansionCtx<'cursor> {
    #[deref]
    #[deref_mut]
    chain_ops: ChainCtx<'cursor>,
}
impl<'a> Iterator for ExpansionCtx<'a> {
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
        let first = match trav.insert_or_get_complete(inner_cursor) {
            Ok(Ok(IndexWithPath { index, path })) => {
                *cursor = path;
                index
            },
            Ok(Err(index)) => index,
            Err(ErrorReason::SingleIndex(c)) => c,
            Err(_) => cursor.start_index(&trav),
        };

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
        //if self.bundle.len() > 1 {
        //    self.bundle.wrap_into_band(&mut self.ctx.trav.graph);
        //}

        // finish back context
        let link = self.create_expansion_link(&exp);
        let &root = self.cursor_root_index();

        // back context
        // what is the back context?
        // The back context is the complement of the next expansion
        // in the current root index.
        // It is the part of the root index that is not covered by the next expansion.
        // It is used to create a new band that will be appended to the chain.
        // The back context is used to create a new band that will be appended to the chain
        // and to create a new expansion link that will be used to link the new band
        // to the previous band in the chain.
        //
        // what is the expansion link?
        // The expansion link is the link between the new band and the previous band in the chain
        // It is used to link the new band to the previous band in the chain.
        // It contains the prefix path, the expansion and the start bound.
        // The prefix path is the path from the start of the root index to the start of
        // the next expansion.
        //
        // what is a band?
        // A band is a collection of indices that are adjacent to each other. It has a pattern,
        // a start bound and an end bound.
        //
        // what is a band chain?
        // A band chain is a collection of bands that are ordered by their end bound.
        // It is used to keep track of the bands that have been created so far and to finally
        // create a final index that contains all the bands in the chain.
        //
        // Adding an expansion to the chain:
        // 1.
        let complement = ComplementBuilder::new(link).build(&mut self.trav);

        //BandExpansion
        //let start_bound = exp.start_bound();
        //let end_bound = exp.start_bound() + index.width();

        self.chain.append_front_complement(complement, exp);
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
