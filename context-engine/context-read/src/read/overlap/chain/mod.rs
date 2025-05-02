use std::{
    borrow::Borrow,
    collections::{
        BTreeMap,
        BTreeSet,
        VecDeque,
    },
};

use derivative::Derivative;
use derive_more::derive::{
    Deref,
    DerefMut,
};

use context_trace::{
    graph::vertex::{
        child::Child,
        pattern::Pattern,
        wide::Wide,
    },
    path::{
        accessors::role::{
            End,
            Start,
        },
        structs::{
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::IndexRoot,
            },
        },
    },
    trace::has_graph::HasGraphMut,
};

use band::{
    Band,
    BandCtx,
    Overlap,
};

use super::{
    bundle::Bundle,
    generator::ExpansionLink,
};
pub mod band;

/// IMPORTANT:
/// - Use OverlapLinks to build SplitVertices
/// - yield postfixes when joining partition for front context of each band
/// - Every new overlap:
///     - complete back context with interval graph from postfix paths
/// - every time a band is "surpassed" (next.start_bound >= band.end_bound)
///     - complete front context with interval graph from prefix paths
///
/// BandChain (building bundle)
///
/// - list of OverlapBand (Pattern with extra information about last index)
///     - start_bound, end_bound of last index
/// - each band is completed up to its end_bound
/// - bands ordered by start_bound (end_bound required to be at least previous start_bound)
///
/// - list of OverlapLink
///     - path to overlap from two bands
///         - postfix_path: location of postfix/overlap in prev band
///         - prefix_path: location of prefix/overlap in next band
///
/// - append new band
///     - if start_bound < some previous end_bound
///         1. describe back context partition between first band and new index with IntervalGraph
///             1. get overlap paths from each band to new index
///             2. build IntervalGraph
///             3. join partition
///             4. find front context postfix for each band
///
///     - if start_bound = some previous end_bound
///         1. bundle chain up to end_bound
///         2. reduce bundle to band
///         3. append index at start_bound
///         4. append band to end of chain
///
/// - get info about latest band
///
/// - bundle chain (take a chain and bake it into a bundle)
///     - describe postfix partition of latest band with IntervalGraph
///         1. get overlap paths from each band to new index
///         2. build IntervalGraph
///         3. join partition
///         4. find front context postfix for each band
///     - join and merge postfix partition
///     - complete each band with front context
///     - if multiple bands, insert new bundle
///     
/// IntervalBuilder
/// - build from list of OverlapLink
/// - build from list of RolePath<R>
/// - build from FoldState
/// - build complete IntervalGraph
///
/// PatternBundle (completed bundle)
/// - list of Pattern
/// - reduce to one pattern by inserting into graph if necessary
///
/// OverlapBand
/// - Pattern
/// - PostfixInfo
///
/// - append new postfix
///
/// PostfixInfo
/// - start_bound, end_bound
///
/// OverlapLink
/// - postfix_path
/// - prefix_path
///
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
#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: RolePath<Start>, // location of prefix/overlap in second index
}
#[derive(Default, Clone, Debug)]
pub struct OverlapChain {
    pub bands: BTreeSet<Band>,
    pub links: VecDeque<OverlapLink>,
}
impl OverlapChain {
    pub fn new(index: Child) -> Self {
        Self {
            bands: Some(Band {
                pattern: vec![index],
                start_bound: 0,
                end_bound: index.width(),
            })
            .into_iter()
            .collect(),
            links: Default::default(),
        }
    }
    pub fn ends_at(
        &self,
        bound: usize,
    ) -> Option<BandCtx<'_>> {
        let band = self.bands.get(&bound)?;

        Some(BandCtx {
            band,
            back_link: self.links.iter().last(),
            front_link: None,
        })
    }
    pub fn last(&self) -> BandCtx<'_> {
        let band = self.bands.iter().last().unwrap();
        BandCtx {
            band,
            back_link: self.links.iter().last(),
            front_link: None,
        }
    }
    pub fn append(
        &mut self,
        band: impl ChainAppendage,
    ) {
        band.append_to_chain(self);
    }
    pub fn pop_first(&mut self) -> Option<Band> {
        self.links.pop_front();
        self.bands.pop_first()
    }

    //pub fn append_overlap(
    //    &mut self,
    //    overlap: OverlapBand,
    //) -> Result<(), ()> {
    //}
    //pub fn take_past(
    //    &mut self,
    //    bound: usize,
    //) -> OverlapChain {
    //    let mut past = self.chain.split_off(&bound);
    //    std::mem::swap(&mut self.chain, &mut past);
    //    Self {
    //        chain: past,
    //        end_bound: bound,
    //    }
    //}

    //#[instrument(skip(self, trav))]
    //pub fn close(
    //    self,
    //    trav: impl HasGraphMut,
    //) -> Option<Child> {
    //    //println!("closing {:#?}", self);
    //    let mut chain_iter = self.chain.into_iter();
    //    let first_band: Overlap = chain_iter.next()?.1;
    //    // this part should create the missing front contexts of each band.
    //    // this should however start at the end of the chain and work backwards
    //    // we need to get paths to the overlaps with each previous band
    //    // then we can use each of these paths to create a partition for the front context of the overlap within the last index
    //    // this can be appended to the back of the band
    //    //
    //    let (mut bundle, prev_band, _) = {
    //        chain_iter.fold(
    //            (vec![], first_band, None),
    //            |(mut bundle, prev_band, prev_ctx), (_end_bound, overlap)| {
    //                // index context of prefix
    //                let ctx_child = if let Some(link) = overlap.link.as_ref() {
    //                    todo!("implement front context indexing");
    //                    //reader
    //                    //    .contexter::<SplitFront>()
    //                    //    .try_context_path(
    //                    //        link.prefix_path
    //                    //            .get_path()
    //                    //            .unwrap()
    //                    //            .clone()
    //                    //            .into_context_path(),
    //                    //        //node.overlap,
    //                    //    )
    //                    //    .map(|(ctx, _)| ctx)
    //                    None
    //                } else {
    //                    None
    //                };

    //                bundle.push(prev_band);
    //                (
    //                    bundle,
    //                    overlap,
    //                    // join previous and current context into
    //                    prev_ctx
    //                        .map(|prev| {
    //                            ctx_child
    //                                .map(|ctx_child| {
    //                                    trav.read_pattern(vec![prev, ctx_child]).unwrap()
    //                                })
    //                                .or(prev)
    //                        })
    //                        .or(ctx_child),
    //                )
    //            },
    //        )
    //    };
    //    bundle.push(prev_band);
    //    let bundle = bundle
    //        .into_iter()
    //        .map(|overlap| overlap.band.into_pattern())
    //        .collect_vec();
    //    let index = trav.graph_mut().insert_patterns(bundle);
    //    //println!("close result: {:?}", index);
    //    Some(index)
    //}

    //pub fn take_past_context_pattern(
    //    &mut self,
    //    trav: impl HasGraphMut,
    //    start_bound: usize,
    //) -> Option<(usize, Pattern)> {
    //    let mut past = self.take_past(start_bound);
    //    match past.chain.len() {
    //        0 => None,
    //        1 => {
    //            let (end_bound, past) = past.chain.pop_last().unwrap();
    //            Some((end_bound, past.band.into_pattern()))
    //        }
    //        _ => {
    //            let past_key = *past.chain.keys().last().unwrap();
    //            let past_index = past.close(trav).unwrap();
    //            Some((past_key, vec![past_index]))
    //        }
    //    }
    //}
}
