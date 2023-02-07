use crate::*;
use super::*;

pub type Folder<Ty>
    = <Ty as DirectedTraversalPolicy>::Trav;

pub trait TraversalFolder<
    S: DirectedTraversalPolicy<Trav=Self>,
>: Sized + Traversable {
    type NodeCollection: NodeCollection;

    //fn map_state(
    //    &self,
    //    acc: ControlFlow<Self::Break, Self::Continue>,
    //    node: TraversalState<R, Q>
    //) -> ControlFlow<Self::Break, Self::Continue>;

    fn fold_query<P: IntoPattern>(
        &self,
        query_pattern: P,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)> {
        let query_pattern = query_pattern.into_pattern();
        let query = QueryState::new::<<Self::Kind as GraphKind>::Direction, _>(query_pattern.borrow())
            .map_err(|(err, q)| (err, q.to_rooted(query_pattern.clone())))?;
        let index = query.clone().to_rooted(query_pattern.clone()).role_leaf_child::<End, _>(self);

        let start = StartState {
            index,
            query: query.clone(),
        };
        let mut states = OrderedTraverser::<_, _, Self::NodeCollection>::new(self);
        let (start_key, mut cache) = TraversalCache::new(&start, query_pattern.clone());
        states.extend(
            states.query_start(start_key, query.clone().to_cached(&mut cache), start)
                .into_states()
                .into_iter()
                .map(|n| (1, n))
        );
        let mut end_states = vec![];

        while let Some((depth, next_states)) = states.next_states(&mut cache) {
            match next_states {
                NextStates::End(next) => {
                    if next.matched {
                        if matches!(next.inner.kind, EndKind::Range(_)) {
                            // stop other paths not with this root
                            //states.prune_not_below(state.root_key());
                        }
                        if let Some(root_key) = next.inner.waiting_root_key() {
                            states.extend(
                                cache.continue_waiting(&root_key)
                            );
                        }
                        end_states.push(next.inner);
                    } else {
                        // stop other paths with this root
                        states.prune_below(next.inner.root_key());
                    }
                },
                _ => {
                    states.extend(
                        next_states.into_states()
                            .into_iter()
                            .map(|nstate| (depth, nstate))
                    );
                },
            }
        }
        Ok(if end_states.is_empty() {
            TraversalResult {
                query: query.to_rooted(query_pattern),
                path: FoundPath::Complete(index)
            }
        } else {

            // todo: find single root and end state
            let final_states = end_states.into_iter()
                .map(|s|
                    FinalState {
                        num_parents: cache.get_entry(&s.root_key())
                            .unwrap()
                            .num_parents(),
                        state: s,
                    }
                )
                .sorted() 
                .collect_vec();
            //println!(
            //    "#############\n{:#?}",
            //    cache.entries.values().map(|e|
            //        (
            //            e.index, e.top_down.keys().map(|prev|
            //                PrevDir::TopDown(prev.index)
            //            ).chain(
            //                e.bottom_up.keys().map(|prev|
            //                    PrevDir::BottomUp(prev.index)
            //                )
            //            )
            //            .collect_vec()
            //        )
            //    )
            //    .collect_vec()
            //);
            let fin = &final_states.first().unwrap();
            let query = fin.state.query.clone();
            let found_path = if let EndKind::Complete(c) = &fin.state.kind {
                    FoundPath::Complete(*c)
                } else {
                    FoundPath::Path(FoldResult {
                        cache,
                        final_states
                    })
                };
            TraversalResult {
                query: query.to_rooted(query_pattern),
                path: found_path,
            }
        })
    }
}
#[derive(Debug)]
pub enum PrevDir {
    TopDown(Child),
    BottomUp(Child),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FinalState {
    pub num_parents: usize,
    pub state: EndState,
}
impl PartialOrd for FinalState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FinalState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num_parents.cmp(&other.num_parents)
            .then_with(||
                other.state.is_complete().cmp(&self.state.is_complete())
                    .then_with(||
                        other.state.root_key().width().cmp(&self.state.root_key().width())
                    )
            )
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldResult {
    pub cache: TraversalCache,
    pub final_states: Vec<FinalState>,
}
impl FoldResult {
    pub fn root_entry(&self) -> &PositionCache {
        self.cache.entries.get(&self.root_index().index()).unwrap()
    }
    pub fn root_index(&self) -> Child {
        // assert same root
        let root = self.final_states.first().unwrap().state.root_key().index;
        assert!(
            self.final_states
                .iter()
                .skip(1)
                .all(|s|
                    s.state.root_key().index == root
                )
        );
        root
    }
    pub fn into_found_path(self) -> FoundPath {
        //todo!("handle complete");
        FoundPath::Path(self)
    }
}