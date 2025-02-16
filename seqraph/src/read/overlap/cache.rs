use std::ops::ControlFlow;

use crate::read::{
    bundle::OverlapBundle,
    context::HasReadContext,
    overlap::chain::OverlapChain,
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

use super::{
    iterator::OverlapIterator,
    match_end::MatchEnd,
};

#[derive(Default, Clone, Debug)]
pub struct OverlapCache {
    pub chain: OverlapChain,
}

impl OverlapCache {
    pub fn new(first: Child) -> Self {
        Self {
            chain: OverlapChain {
                last: Overlap {
                    link: None,
                    band: OverlapBand::from(first),
                }
                .into(),
                end_bound: first.width(),
                chain: Default::default(),
            },
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
