use crate::read::{
    context::ReadContext,
    overlap::chain::OverlapChain,
};
use context_insert::insert::ToInsertContext;
use context_search::traversal::iterator::bands::{
    BandIterator,
    PostfixIterator,
    PrefixIterator,
};
use context_trace::{
    graph::vertex::{
        child::Child,
        wide::Wide,
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
            role_path::RolePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::{
                    PatternEndPath,
                    RootedRolePath,
                },
            },
            sub_path::SubPath,
        },
    },
};
use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::{
    FoldWhile,
    Itertools,
};

use super::band::Band;

pub trait ChainAppendage: Sized {
    fn append_to_chain(
        self,
        chain: &mut OverlapChain,
    ) {
        chain.bands.insert(self.into_band());
    }
    fn into_band(self) -> Band;
}
impl ChainAppendage for Band {
    fn into_band(self) -> Band {
        self
    }
}
impl ChainAppendage for (usize, Band) {
    fn into_band(self) -> Band {
        self.1
    }
}
#[derive(Debug, From)]
pub enum ChainOp {
    Expansion(ExpansionLink),
    Cap(BandCap),
}
#[derive(Debug)]
pub struct BandCap {
    pub postfix_path: RolePath<End>,
    pub expansion: Child,
    pub start_bound: usize,
}
#[derive(Debug)]
pub struct ExpansionLink {
    pub prefix_path: RolePath<Start>,
    pub expansion: Child,
    pub start_bound: usize,
}

#[derive(Debug, Deref, DerefMut)]
pub struct ChainGenerator<'a> {
    pub trav: ReadContext,
    pub cursor: &'a mut PatternEndPath,
    #[deref]
    #[deref_mut]
    pub chain: OverlapChain,
}

impl<'a> Iterator for ChainGenerator<'a> {
    type Item = ChainOp;

    fn next(&mut self) -> Option<Self::Item> {
        self.find_next_expansion()
    }
}
impl<'a> ChainGenerator<'a> {
    pub fn new(
        trav: ReadContext,
        cursor: &'a mut PatternEndPath,
        chain: OverlapChain,
    ) -> Self {
        Self {
            trav,
            cursor,
            chain,
        }
    }

    /// find largest expandable postfix
    fn find_next_expansion(&mut self) -> Option<ChainOp> {
        let last = self.chain.last();
        let last_end_bound = last.band.end_bound;
        let last_end = last.band.postfix();

        // TODO: Replace with Child Trace Iter
        let mut postfix_iter =
            PostfixIterator::band_iter(self.trav.clone(), last_end);
        let mut postfix_path = {
            let postfix_location = postfix_iter.next().unwrap().0;
            RolePath::from(SubPath::new(postfix_location.sub_index))
        };
        postfix_iter.find_map(|(postfix_location, postfix)| {
            let cursor: PatternEndPath = self.cursor.clone();
            let start_bound = last_end_bound - postfix.width();

            // build path to this location
            postfix_path.path_append(postfix_location);
            match self.trav.insert_or_get_complete(primer) {
                Ok(expansion) => {
                    let link = self.link_expansion(
                        start_bound,
                        expansion,
                        postfix_path,
                    );
                    Some(ChainOp::Expansion(link))
                },
                Err(_) => match self.chain.ends_at(start_bound) {
                    Some(_) => Some(ChainOp::Cap(BandCap {
                        postfix_path: postfix_path.clone(),
                        expansion: postfix,
                        start_bound,
                    })),
                    _ => None,
                },
            }
        })
    }
    fn link_expansion(
        &mut self,
        start_bound: usize,
        expansion: Child,
        advanced: PatternRangePath,
    ) -> ExpansionLink {
        let adv_prefix =
            PatternRootChild::<Start>::pattern_root_child(advanced);
        // find prefix from advanced path in expansion index
        let mut prefix_iter =
            PrefixIterator::band_iter(self.trav.clone(), expansion);
        let entry = prefix_iter.next().unwrap().0;
        let mut prefix_path = prefix_iter
            .fold_while(
                RootedRolePath::new(entry),
                |mut acc, (prefix_location, prefix)| {
                    acc.path_append(prefix_location);
                    if prefix == adv_prefix {
                        FoldWhile::Done(acc)
                    } else {
                        FoldWhile::Continue(acc)
                    }
                },
            )
            .into_inner();
        // append path <expansion to adv_prefix> to <adv_prefix to overlap>
        prefix_path.role_path.sub_path.path.extend(
            (advanced.role_path().clone() as RolePath<End>)
                .sub_path
                .path,
        );

        ExpansionLink {
            start_bound,
            prefix_path: prefix_path.role_path,
            expansion,
        }
    }
}
