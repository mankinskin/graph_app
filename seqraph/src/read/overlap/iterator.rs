use std::ops::ControlFlow;

use crate::read::{
    bundle::OverlapBundle,
    context::{
        HasReadContext,
        ReadContext,
    },
    overlap::chain::OverlapChain,
};
use derive_new::new;
use hypercontext_api::{
    direction::{
        pattern::PatternDirection,
        Direction,
    },
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
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                pattern_prefix::PatternPrefixPath,
                pattern_range::PatternRangePath,
                role_path::RootedRolePath,
            },
            sub_path::SubPath,
        },
    },
    traversal::{
        iterator::bands::{
            BandIterator,
            PostfixIterator,
            PrefixIterator,
        },
        traversable::TravDir,
    },
};
use itertools::{
    FoldWhile,
    Itertools,
};
use petgraph::visit::Control;
use tracing::instrument;

use super::{
    chain::OverlapLink,
    match_end::MatchEnd,
};

pub struct NextOverlap {
    pub start_bound: usize,
    pub link: OverlapLink,
    pub expansion: Child,
}

#[derive(Debug)]
pub struct ReadStateIterator<'a> {
    overlap_iter: OverlapIterator<'a>,
}

impl<'a> Iterator for ReadStateIterator<'a> {
    type Item = Child;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next_overlap()
    }
}
impl<'a> ReadStateIterator<'a> {
    pub fn new(
        trav: ReadContext,
        cursor: &'a mut PatternPrefixPath,
    ) -> Self {
        Self {
            overlap_iter: OverlapIterator::new(trav, cursor),
        }
    }
    pub fn read_next_overlap(&mut self) -> Option<Child> {
        // find expandable postfix, may append postfixes in overlap chain
        //println!("read next overlap with {:#?}", cache.last);
        match self.overlap_iter.next() {
            Some(next) => {
                //println!("found overlap at {}: {:?}", start_bound, expansion);
                //
                let band = self.bundle.wrap_into_band(self.overlap_iter.trav);
                self.chain.append_overlap(Overlap { link: None, band });

                let back_pattern = self.chain.back_context_for_link(
                    &mut self.overlap_iter.trav,
                    next.start_bound,
                    &next.link,
                );

                self.chain.append(
                    next.start_bound,
                    Overlap {
                        band: OverlapBand {
                            end: next.expansion,
                            back_context: back_pattern,
                        },
                        link: Some(next.link), // todo
                    },
                );

                self.read_next_overlap()
            }
            None => {
                self.chain.append_bundle(&mut trav, self.bundle);
                self.chain.close(trav)
            }
        }
    }
}

#[derive(Debug)]
pub struct OverlapIterator<'a> {
    trav: ReadContext,
    cursor: &'a mut PatternPrefixPath,
    bundle: OverlapBundle,
}

impl<'a> Iterator for OverlapIterator<'a> {
    type Item = NextOverlap;

    fn next(&mut self) -> Option<Self::Item> {
        self.find_next_overlap()
    }
}

impl<'a> OverlapIterator<'a> {
    pub fn new(
        trav: ReadContext,
        cursor: &'a mut PatternPrefixPath,
    ) -> Self {
        Self {
            trav,
            cursor,
            bundle: OverlapBundle::default(),
        }
    }

    /// find largest expandable postfix
    #[instrument(skip(self))]
    fn find_next_overlap(&mut self) -> Option<NextOverlap> {
        let last = self.chain.last.take().expect("No last overlap to take!");
        let last_end = last.band.end;

        let mut postfix_iter = PostfixIterator::band_iter(self.trav, last_end);
        let mut path = {
            let postfix_location = postfix_iter.next().unwrap().0;
            RolePath::from(SubPath::new(postfix_location.sub_index))
        };
        postfix_iter.find_map(|(postfix_location, postfix)| {
            // build path to this location
            let start_bound = self.chain.end_bound - postfix.width();
            path.path_append(postfix_location);
            let primer = self.cursor.clone().to_range_path();
            self.expand_postfix(postfix, start_bound, path, primer)
        })
    }
    fn expand_postfix(
        &mut self,
        postfix: Child,
        start_bound: usize,
        postfix_path: RolePath<End>,
        primer: PatternRangePath,
    ) -> Option<NextOverlap> {
        // try expand
        //let primer = OverlapPrimer::new(postfix, cursor.clone());
        let read_overlap = self.trav.read_one(primer);
        match read_overlap {
            Ok((expansion, advanced)) => {
                Some(self.link_expansion(start_bound, postfix_path, expansion, advanced))
            }
            Err(_reason) => {
                // if not expandable, at band boundary -> add to bundle
                // postfixes should always be first in the chain
                if let Some(overlap) = self.chain.remove(&start_bound).map(|band| {
                    // might want to pass postfix_path
                    band.appended(postfix)
                }) {
                    self.bundle.add_band(overlap.band)
                }
                None
            }
        }
    }
    fn link_expansion(
        &mut self,
        start_bound: usize,
        postfix_path: RolePath<End>,
        expansion: Child,
        advanced: PatternRangePath,
    ) -> NextOverlap {
        let adv_prefix = PatternRootChild::<Start>::pattern_root_child(&advanced);
        // find prefix from advanced path in expansion index
        let mut prefix_iter = PrefixIterator::band_iter(self.trav.clone(), expansion);
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

        let link = OverlapLink {
            postfix_path,
            prefix_path: MatchEnd::Path(prefix_path),
        };
        NextOverlap {
            start_bound,
            link,
            expansion,
        }
    }
}
