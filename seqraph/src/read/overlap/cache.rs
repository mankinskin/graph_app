use std::ops::ControlFlow;

use crate::{
    insert::HasInsertContext,
    read::{
        bundle::band::{
            BandEnd,
            OverlapBand,
            OverlapBundle,
        },
        context::HasReadContext,
        overlap::{
            chain::OverlapChain,
            Overlap,
        },
    },
};
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
            match_end::MatchEnd,
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
use tracing::instrument;

use super::OverlapLink;

#[derive(Default, Clone, Debug)]
pub struct OverlapCache {
    pub chain: OverlapChain,
    pub end_bound: usize,
    pub last: Option<Overlap>,
}

impl OverlapCache {
    pub fn new(first: Child) -> Self {
        Self {
            end_bound: first.width(),
            last: Overlap {
                link: None,
                band: OverlapBand::from(first),
            }
            .into(),
            chain: OverlapChain::default(),
        }
    }
    pub fn append(
        &mut self,
        start_bound: usize,
        overlap: Overlap,
    ) {
        let width = overlap.band.end.index().unwrap().width();
        if let Some(last) = self.last.replace(overlap) {
            self.chain.add_overlap(self.end_bound, last).unwrap()
        }
        self.end_bound = start_bound + width;
    }
    #[instrument(skip(self, trav, cursor))]
    pub fn read_next_overlap<Trav: HasReadContext>(
        mut self,
        mut trav: Trav,
        cursor: &mut PatternPrefixPath,
    ) -> Option<Child>
    where
        <TravDir<Trav> as Direction>::Opposite: PatternDirection,
    {
        // find expandable postfix, may append postfixes in overlap chain
        //println!("read next overlap with {:#?}", cache.last);
        match self.find_next_overlap(&mut trav, cursor) {
            (Some((start_bound, next_link, expansion)), bundle) => {
                //println!("found overlap at {}: {:?}", start_bound, expansion);
                //
                self.chain.add_bundle(&mut trav, self.end_bound, bundle);
                let back = self
                    .chain
                    .back_context_for_link(&mut trav, start_bound, &next_link);

                self.append(
                    start_bound,
                    Overlap {
                        band: OverlapBand {
                            end: BandEnd::Index(expansion),
                            back_context: back,
                        },
                        link: Some(next_link), // todo
                    },
                );

                self.read_next_overlap(trav, cursor)
            }
            (None, bundle) => {
                self.chain.add_bundle(&mut trav, self.end_bound, bundle);
                //println!("No overlap found");
                self.chain.close(trav)
            }
        }
    }

    /// find largest expandable postfix
    #[instrument(skip(self, trav, cursor))]
    fn find_next_overlap<Trav: HasReadContext>(
        &mut self,
        mut trav: Trav,
        cursor: &mut PatternPrefixPath,
    ) -> (Option<(usize, OverlapLink, Child)>, OverlapBundle)
    where
        <TravDir<Trav> as Direction>::Opposite: PatternDirection,
    {
        let last = self.last.take().expect("No last overlap to take!");
        let last_end = *last.band.end.index().unwrap();
        let mut postfix_iter = PostfixIterator::band_iter(&mut trav, last_end);

        let mut acc = ControlFlow::Continue((
            None as Option<RolePath<End>>,
            OverlapBundle::from(last.band),
        ));
        while let (true, Some((postfix_location, postfix))) =
            (acc.is_continue(), postfix_iter.next())
        {
            let (path, bundle) = acc.continue_value().unwrap();
            let start_bound = self.end_bound - postfix.width();
            let postfix_path = if let Some(mut path) = path {
                path.path_append(postfix_location);
                path
            } else {
                RolePath::from(SubPath::new(postfix_location.sub_index))
            };

            let primer = cursor.clone().to_range_path();
            acc = match self.expand_postfix(
                postfix_iter.trav_mut(),
                postfix,
                start_bound,
                bundle,
                postfix_path,
                primer,
            ) {
                ControlFlow::Break((start_bound, next, expansion, bundle)) => {
                    ControlFlow::Break((Some((start_bound, next, expansion)), bundle))
                }
                ControlFlow::Continue((_, bundle)) => ControlFlow::Continue((None, bundle)),
            };
        }
        match acc {
            ControlFlow::Break(val) => val,
            ControlFlow::Continue((_, b)) => (None, b),
        }
    }
    fn expand_postfix(
        &mut self,
        mut trav: impl HasReadContext,
        postfix: Child,
        start_bound: usize,
        mut bundle: OverlapBundle,
        postfix_path: RolePath<End>,
        primer: PatternRangePath,
    ) -> ControlFlow<
        (usize, OverlapLink, Child, OverlapBundle),
        (Option<RolePath<End>>, OverlapBundle),
    > {
        // try expand
        //let primer = OverlapPrimer::new(postfix, cursor.clone());
        match trav.read_context().read_one(primer) {
            Ok((expansion, advanced)) => {
                let adv_prefix = PatternRootChild::<Start>::pattern_root_child(&advanced);
                // find prefix from advanced path in expansion index
                let mut prefix_iter = PrefixIterator::band_iter(&trav, expansion);
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
                ControlFlow::Break((start_bound, link, expansion, bundle))
            }
            Err(_) => {
                // if not expandable, at band boundary -> add to bundle
                // postfixes should always be first in the chain
                if let Some(overlap) = self.chain.remove(&start_bound).map(|band| {
                    // might want to pass postfix_path
                    band.appended(BandEnd::Index(postfix))
                }) {
                    bundle.add_band(overlap.band)
                }
                ControlFlow::Continue((Some(postfix_path), bundle))
            }
        }
    }
    //// TODO: Is this really needed? (possible?)
    ///// next bands generated when next overlap starts strictly behind the end (with a gap) of a past bundle
    //#[instrument(skip(self, cache, past_end_bound, next_link, expansion, past_ctx))]
    //pub fn odd_overlap_next(
    //    &mut self,
    //    cache: &mut OverlapCache,
    //    past_end_bound: usize,
    //    next_link: &OverlapLink,
    //    expansion: Child,
    //    past_ctx: Pattern,
    //) -> Overlap {
    //    //println!("odd overlap");
    //    let last = cache.last.as_mut().unwrap();
    //    let prev = last.band.end.clone().into_index(self);
    //    last.band.end = BandEnd::Index(prev);
    //    // split last band to get overlap with past bundle
    //    let IndexSplitResult {
    //        inner,
    //        location,
    //        path,
    //    } = self.splitter::<SplitFront>().single_offset_split(
    //        prev,
    //        NonZeroUsize::new(cache.end_bound - past_end_bound).unwrap(),
    //    );
    //    assert!(path.is_empty());
    //
    //    // build new context path (to overlap)
    //    let context_path = {
    //        // entry in last band (could be handled by IndexSplit
    //        let inner_entry = {
    //            let graph = self.graph.graph();
    //            let (pid, pattern) = graph.expect_vertex(inner).expect_any_child_pattern();
    //            ChildLocation {
    //                parent: inner,
    //                pattern_id: *pid,
    //                sub_index: DefaultDirection::last_index(pattern.borrow()),
    //            }
    //        };
    //        // FIXME: maybe mising root!!!
    //        let postfix_path = next_link.postfix_path.clone().sub_path;
    //        Vec::with_capacity(postfix_path.len() + 2).tap_mut(|v| {
    //            v.push(location);
    //            v.push(inner_entry);
    //            v.extend(postfix_path.into_iter().skip(1));
    //        })
    //    };
    //    // get index between past and next overlap
    //    let (inner_back_ctx, _loc) = self
    //        .contexter::<SplitBack>()
    //        .try_context_path(
    //            context_path,
    //            //next_link.overlap,
    //        )
    //        .unwrap();
    //
    //    let past = self.graph.graph_mut().insert_pattern(past_ctx);
    //    let past_inner = self
    //        .graph
    //        .graph_mut()
    //        .insert_pattern([past, inner_back_ctx]);
    //    let inner_expansion = self
    //        .graph
    //        .graph_mut()
    //        .insert_pattern([inner_back_ctx, expansion]);
    //    let index = self
    //        .graph
    //        .graph_mut()
    //        .insert_patterns([[past_inner, expansion], [past, inner_expansion]]);
    //    Overlap {
    //        band: OverlapBand {
    //            end: BandEnd::Index(index),
    //            back_context: vec![],
    //        },
    //        link: None, // todo
    //    }
    //}
}
