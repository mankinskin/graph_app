mod band;
mod cache;
mod chain;
mod context;

pub use {
    band::*,
    cache::*,
    chain::*,
};

use super::*;

impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    #[instrument(skip(self, first, context))]
    pub fn read_overlaps(
        &mut self,
        first: Child,
        context: &mut PrefixQuery,
    ) -> Option<Child> {
        if first.width() > 1 {
            self.read_next_overlap(
                OverlapCache::new(first),
                context,
            )
        } else {
            None
        }
    }
    /// next bands generated when next overlap starts after a past bundle with a gap
    #[instrument(skip(self, cache, past_end_bound, next_link, expansion, past_ctx))]
    pub fn odd_overlap_next(
        &mut self,
        cache: &mut OverlapCache,
        past_end_bound: usize,
        next_link: &OverlapLink,
        expansion: Child,
        past_ctx: Pattern,
    ) -> Overlap {
        //println!("odd overlap");
        let last = cache.last.as_mut().unwrap();
        let prev = last.band.end.clone().into_index(self);
        last.band.end = BandEnd::Index(prev);
        // split last band to get overlap with past bundle
        let IndexSplitResult {
            inner,
            location,
            path,
        } = self.splitter::<IndexFront>().single_offset_split(
            prev,
            NonZeroUsize::new(cache.end_bound - past_end_bound).unwrap(),
        );
        assert!(path.is_empty());

        // build new context path (to overlap)
        let context_path = {
            // entry in last band (could be handled by IndexSplit
            let inner_entry = {
                let graph = self.graph.graph();
                let (pid, pattern) = graph.expect_vertex_data(inner).expect_any_child_pattern();
                ChildLocation {
                    parent: inner,
                    pattern_id: *pid,
                    sub_index: D::last_index(pattern.borrow()),
                }
            };
            let postfix_path = next_link.postfix_path.clone().into_context_path();
            Vec::with_capacity(postfix_path.len() + 2).tap_mut(|v| {
                v.push(location);
                v.push(inner_entry);
                v.extend(postfix_path.into_iter().skip(1));
            })
        };
        // get index between past and next overlap
        let (inner_back_ctx, _loc) = self.contexter::<IndexBack>().try_context_path(
            context_path,
            //next_link.overlap,
        ).unwrap();

        let past = self.graph.graph_mut().insert_pattern(past_ctx);
        let past_inner = self.graph.graph_mut().insert_pattern([past, inner_back_ctx]);
        let inner_expansion = self.graph.graph_mut().insert_pattern([inner_back_ctx, expansion]);
        let index = self.graph.graph_mut().insert_patterns([
            [past_inner, expansion],
            [past, inner_expansion],
        ]);
        Overlap {
            band: OverlapBand {
                end: BandEnd::Index(index),
                back_context: vec![],
            },
            link: None, // todo
        }
    }
    //#[async_recursion]
    #[instrument(skip(self, cache, context))]
    pub fn read_next_overlap(
        &mut self,
        cache: OverlapCache,
        context: &mut PrefixQuery,
    ) -> Option<Child> {
        // find expandable postfix, may append postfixes in overlap chain
        //println!("read next overlap with {:#?}", cache.last);
        match self.find_next_overlap(
                cache,
                context,
            ) {
            Ok((start_bound, next_link, expansion, mut cache)) => {
                    //println!("found overlap at {}: {:?}", start_bound, expansion);
                    if let Some(ctx) =
                        if let Some((past_end_bound, past_ctx)) = self.take_past_context_pattern(
                            start_bound,
                            &mut cache.chain,
                        ) {
                            println!("reusing back context {past_end_bound}: {:#?}", past_ctx);
                            if past_end_bound == start_bound {
                                Some(past_ctx)
                            } else {
                                assert!(past_end_bound < start_bound);
                                let next = self.odd_overlap_next(
                                    &mut cache,
                                    past_end_bound,
                                    &next_link,
                                    expansion,
                                    past_ctx,
                                );
                                cache.append(
                                    self,
                                    start_bound,
                                    next,
                                );

                                //OverlapBand {
                                //    end: BandEnd::Chain(expansion),
                                //    back_context: vec![inner_back_ctx],
                                //}
                                // use postfix_path of next to index inner context in previous
                                // index overlap between previous and new coming index
                                // create new bundle to start with

                                // - remember to bundle latest bands 
                                // - until after band where odd overlap ocurred
                                // - add extra band with postfix of previous band
                                // - if overlap after previous band 
                                None
                            }
                        } else {
                            //println!("building back context from path");
                            Some(self.back_context_from_path(&mut cache.chain, &next_link))
                        }
                {
                    cache.append(
                        self,
                        start_bound,
                        Overlap {
                            band: OverlapBand {
                                end: BandEnd::Index(expansion),
                                back_context: ctx,
                            },
                            link: Some(next_link), // todo
                        }
                    );
                }

                    self.read_next_overlap(
                        cache,
                        context,
                    )
                },
            Err(cache) => {
                //println!("No overlap found");
                cache.chain.close(self)
            }
        }
    }
    /// find largest expandable postfix
    #[instrument(skip(self, cache, prefix_query))]
    fn find_next_overlap(
        &mut self,
        mut cache: OverlapCache,
        prefix_query: &mut PrefixQuery,
    ) -> Result<(usize, OverlapLink, Child, OverlapCache), OverlapCache> {
        let last = cache.last.take().expect("No last overlap to take!");
        let last_end = *last.band.end.index().unwrap();
        let mut acc = ControlFlow::Continue((None as Option<ChildPath>, OverlapBundle::from(last.band)));
        let mut indexer = self.indexer();
        let mut iter = PostfixIterator::<_, D, _>::new(&mut indexer, last_end);
        while let Some((postfix_location, postfix)) = iter.next() {
            let (path, mut bundle) = acc.continue_value().unwrap();
            let start_bound = cache.end_bound - postfix.width();

            let postfix_path = if let Some(path) = path {
                path.append::<_, D, _>(&self.graph, postfix_location)
            } else {
                ChildPath::from(PathLeaf::new(postfix, postfix_location))
            };
            // try expand
            match self.graph
                .index_query_with_origin(OverlapPrimer::new(postfix, prefix_query.clone()))
            {
                Ok((
                    OriginPath {
                        postfix: expansion,
                        origin: prefix_path,
                    },
                    advanced,
                )) => {
                    *prefix_query = advanced.into_prefix_path();

                    acc = ControlFlow::Break((
                        start_bound,
                        OverlapLink {
                            postfix_path,
                            prefix_path,
                        },
                        expansion,
                        bundle,
                    ));
                    break;
                },
                Err(_) => {
                    // if not expandable, at band boundary -> add to bundle
                    // postfixes should always be first in the chain
                    if let Some(overlap) = cache.chain.path.remove(&start_bound).map(|band| {
                        // might want to pass postfix_path
                        band.appended(self, BandEnd::Index(postfix))
                    }) {
                        bundle.add_band(overlap.band)
                    }
                    acc = ControlFlow::Continue((Some(postfix_path), bundle));
                },
            }
        }
        match acc {
            ControlFlow::Break((start_bound, next, expansion, bundle)) => {
                cache.add_bundle(self, bundle);
                Ok((start_bound, next, expansion, cache))
            },
            ControlFlow::Continue((_, bundle)) => {
                cache.add_bundle(self, bundle);
                Err(cache)
            }
        }
    }
}