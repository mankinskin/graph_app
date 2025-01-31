use std::ops::ControlFlow;

use crate::{
    insert::HasInsertContext,
    read::{
        bundle::band::{
            BandEnd,
            OverlapBand,
            OverlapBundle,
        },
        overlap::{
            chain::OverlapChain,
            Overlap,
        },
        reader::context::ReadContext,
    },
};
use hypercontext_api::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::pattern::PatternLocation,
            wide::Wide,
        },
    },
    path::{
        accessors::{
            child::root::PatternRootChild,
            has_path::{
                HasRolePath,
                IntoRootedRolePath,
            },
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
                role_path::RootedRolePath,
                root::IndexRoot,
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
        traversable::TraversableMut,
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
    pub fn add_bundle(
        &mut self,
        reader: &mut ReadContext,
        bundle: OverlapBundle,
    ) {
        self.chain.insert(
            self.end_bound,
            Overlap {
                link: None,
                band: bundle.into_band(reader),
            },
        );
    }
    pub fn append(
        &mut self,
        _reader: &mut ReadContext,
        start_bound: usize,
        overlap: Overlap,
    ) {
        let width = overlap.band.end.index().unwrap().width();
        if let Some(last) = self.last.replace(overlap) {
            self.chain.add_overlap(self.end_bound, last).unwrap()
        }
        self.end_bound = start_bound + width;
    }
    #[instrument(skip(self, ctx, cursor))]
    pub fn read_next_overlap(
        mut self,
        ctx: &mut ReadContext,
        cursor: &mut PatternPrefixPath,
    ) -> Option<Child> {
        // find expandable postfix, may append postfixes in overlap chain
        //println!("read next overlap with {:#?}", cache.last);
        match self.find_next_overlap(ctx, cursor) {
            Some((start_bound, next_link, expansion)) => {
                //println!("found overlap at {}: {:?}", start_bound, expansion);
                let past_ctx = ctx.take_past_context_pattern(start_bound, &mut self.chain);
                let pat = if let Some((past_end_bound, past_ctx)) = past_ctx {
                    println!("reusing back context {past_end_bound}: {:#?}", past_ctx);
                    if past_end_bound == start_bound {
                        Some(past_ctx)
                    } else {
                        assert!(past_end_bound < start_bound);
                        panic!("Shouldn't this be impossible?!");
                    }
                } else {
                    //println!("building back context from path");
                    Some(ctx.back_context_from_path(&mut self.chain, &next_link))
                };
                if let Some(pat) = pat {
                    self.append(
                        ctx,
                        start_bound,
                        Overlap {
                            band: OverlapBand {
                                end: BandEnd::Index(expansion),
                                back_context: pat,
                            },
                            link: Some(next_link), // todo
                        },
                    );
                }

                self.read_next_overlap(ctx, cursor)
            }
            None => {
                //println!("No overlap found");
                self.chain.close(ctx)
            }
        }
    }
    /// find largest expandable postfix
    #[instrument(skip(self, ctx, cursor))]
    fn find_next_overlap(
        &mut self,
        ctx: &mut ReadContext,
        cursor: &mut PatternPrefixPath,
    ) -> Option<(usize, OverlapLink, Child)> {
        let last = self.last.take().expect("No last overlap to take!");
        let last_end = *last.band.end.index().unwrap();

        let mut acc = ControlFlow::Continue((
            None as Option<RolePath<End>>,
            OverlapBundle::from(last.band),
        ));

        let mut insert_context = ctx.insert_context();
        let mut iter = PostfixIterator::band_iter(&mut insert_context.graph_mut(), last_end);

        while let Some((postfix_location, postfix)) = iter.next() {
            let (path, mut bundle) = acc.continue_value().unwrap();
            let start_bound = self.end_bound - postfix.width();

            let postfix_path = if let Some(path) = path {
                path.path_append(postfix_location);
                path
            } else {
                RolePath::from(SubPath::new(postfix_location.sub_index))
            };
            // try expand
            //let primer = OverlapPrimer::new(postfix, cursor.clone());
            let primer = cursor.clone().to_range_path();
            match ctx.graph.insert_context().insert(primer) {
                Ok((expansion, advanced)) => {
                    let adv_prefix = PatternRootChild::<Start>::pattern_root_child(&advanced);
                    // find prefix from advanced path in expansion index
                    let prefix_iter = PrefixIterator::band_iter(&ctx.graph, expansion);
                    let entry = prefix_iter.next().unwrap().0;
                    let mut prefix_path = prefix_iter
                        .fold_while(
                            RootedRolePath::new(entry),
                            |acc, (prefix_location, prefix)| {
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
                    prefix_path
                        .role_path
                        .sub_path
                        .extend(advanced.role_path().sub_path.path);
                    let link = OverlapLink {
                        postfix_path,
                        prefix_path: MatchEnd::Path(prefix_path),
                    };
                    acc = ControlFlow::Break((start_bound, link, expansion, bundle));
                    break;
                }
                Err(_) => {
                    // if not expandable, at band boundary -> add to bundle
                    // postfixes should always be first in the chain
                    if let Some(overlap) = self.chain.remove(&start_bound).map(|band| {
                        // might want to pass postfix_path
                        band.appended(ctx, BandEnd::Index(postfix))
                    }) {
                        bundle.add_band(overlap.band)
                    }
                    acc = ControlFlow::Continue((Some(postfix_path), bundle));
                }
            }
        }
        match acc {
            ControlFlow::Break((start_bound, next, expansion, bundle)) => {
                self.add_bundle(ctx, bundle);
                Some((start_bound, next, expansion))
            }
            ControlFlow::Continue((_, bundle)) => {
                self.add_bundle(ctx, bundle);
                None
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
