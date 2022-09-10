use crate::*;
use super::*;

pub(crate) struct OverlapNode {
    end_bound: usize,
    back_context: Pattern,
    index: Child,
}

impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn read_overlaps(
        &mut self,
        first: Child,
        context: &mut PrefixPath,
    ) -> Option<Child> {
        if first.width() < 2 {
            assert!(first.width() != 0);
            return None
        }
        self.read_next_overlap(
            OverlapBands::new(first),
            OverlapNode {
                end_bound: first.width(),
                back_context: vec![],
                index: first,
            },
            context,
        )
    }
    pub(crate) fn read_next_overlap(
        &mut self,
        mut bands: OverlapBands,
        node: OverlapNode,
        context: &mut PrefixPath,
    ) -> Option<Child> {
        let end_bound = node.end_bound;
        if let Some(node) = self.find_node_overlap(
            &mut bands,
            node,
            context,
        ) {
            self.read_next_overlap(
                bands,
                node,
                context,
            )
        } else {
            self.close_bands(bands, end_bound)
        }
    }
    /// finds the earliest overlap
    fn find_node_overlap(
        &mut self,
        bands: &mut OverlapBands,
        node: OverlapNode,
        prefix_path: &mut PrefixPath,
    ) -> Option<OverlapNode> {
        let OverlapNode {
            end_bound,
            back_context,
            index,
        } = node;
        // find largest expandable postfix
        let mut path = vec![];
        // bundles of bands with same lengths for new index
        PostfixIterator::<_, D, _>::new(&self.indexer(), index)
            .find_map(|(parent_location, loc, postfix)| {
                if let Some(segment) = parent_location {
                    path.push(segment);
                }
                let start_bound = end_bound - postfix.width();
                // if at band boundary and bundle exists, index band
                let band = bands.take_min_past_band(&mut self.indexer(), start_bound)
                    .map(|(end_bound, band)|
                        match end_bound.cmp(&start_bound) {
                            Ordering::Equal => band,
                            Ordering::Less => {
                                band
                                //let mut graph = self.indexer().graph_mut();
                                //graph.index_range_in(loc, D::front_context_range(loc.sub_index))
                            },
                            Ordering::Greater => panic!("not a past band!"),
                        }
                    );
                // expand
                match self.graph.index_query(OverlapPrimer::new(postfix, prefix_path.clone()))
                    .map(|(expansion, advanced)| {
                        *prefix_path = advanced.into_prefix_path();
                        expansion
                    }) {
                        Ok(expansion) => {
                            let next_bound = start_bound + expansion.width();
                            let node =
                                OverlapNode {
                                    end_bound: next_bound,
                                    back_context: band.unwrap_or_else(||
                                        // create band with generated back context
                                        vec![
                                            SideIndexable::<T, D, IndexBack>::context_path(
                                                self,
                                                loc,
                                                path.clone(),
                                                postfix,
                                            ).0,
                                        ]
                                    ),
                                    index: expansion,
                                };
                            bands.add_band(
                                node.end_bound,
                                node.back_context.clone().tap_mut(|b| b.push(node.index)),
                            );
                            Some(node)
                        },
                        Err(_) => {
                            // if not expandable, at band boundary and no bundle exists, create bundle
                            if let Some(mut band) = band {
                                band.push(postfix);
                                bands.insert(
                                    end_bound,
                                    vec![
                                        back_context.clone().tap_mut(|b| b.push(index)),
                                        band,
                                    ]
                                );
                            }
                            None
                        },
                    }
            })
    }
    fn close_bands(
        &mut self,
        mut bands: OverlapBands,
        end_bound: usize,
    ) -> Option<Child> {
        (!bands.is_empty()).then(|| {
            let bundle = bands.remove(&end_bound).expect("Bands not finished at end_bound!");
            assert!(bands.is_empty(), "Bands not finished!");
            self.indexer().graph_mut().index_patterns(bundle)
        })
    }
}