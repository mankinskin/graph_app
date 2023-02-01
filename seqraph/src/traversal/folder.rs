use crate::*;
use super::*;

pub type Folder<R, Ty>
    = <Ty as DirectedTraversalPolicy<R>>::Trav;

pub type FolderPathPair<R>
    = PathPair<<R as ResultKind>::Advanced, <R as ResultKind>::Query>;

pub trait TraversalFolder<
    S: DirectedTraversalPolicy<R, Trav=Self>,
    R: ResultKind,
>: Sized + Traversable {
    //type Break;
    //type Continue;
    //type Result = ControlFlow<Self::Break, Self::Continue>;
    type NodeCollection: NodeCollection<R>;

    //fn map_state(
    //    &self,
    //    acc: ControlFlow<Self::Break, Self::Continue>,
    //    node: TraversalState<R, Q>
    //) -> ControlFlow<Self::Break, Self::Continue>;

    fn fold_query<P: IntoPattern>(
        &self,
        query: P,
    ) -> Result<TraversalResult<R>, (NoMatch, R::Query)> {
        let query_path = R::Query::new_directed::<<Self::Kind as GraphKind>::Direction, _>(query.borrow())
            .map_err(|(err, q)| (err, q))?;
        let index = query_path.leaf_child(self);

        let start = StartState::new(index, query_path.clone());
        let mut states = OrderedTraverser::<_, _, _, Self::NodeCollection>::new(self);
        let (start_key, mut cache) = TraversalCache::new(&start);
        states.extend(
            states.query_start(start_key, start)
                .into_states()
                .into_iter()
                .map(|n| (1, n))
        );
        let mut end_states = vec![];

        while let Some((depth, next_states)) = states.next_states(&mut cache) {
            match next_states {
                NextStates::End(prev, matched, state) => {
                    if matched {
                        if matches!(state.kind, EndKind::Range(_)) {
                            // stop other paths not with this root
                            //states.prune_not_below(state.root_key());
                        }
                        if let Some(root_key) = state.waiting_root_key() {
                            states.extend(
                                cache.continue_waiting(&root_key)
                            );
                        }
                        end_states.push(state);
                    } else {
                        // stop other paths with this root
                        states.prune_below(state.root_key());
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
        if end_states.is_empty() {
            return Ok(TraversalResult {
                query: query_path,
                path: <R as ResultKind>::Found::from(
                    FoundPath::Complete(index)
                )
            });
        } else {
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
            let fin = &final_states.last().unwrap();
            Ok(TraversalResult {
                query: fin.state.query.clone(),
                path: <R as ResultKind>::Found::from(if let EndKind::Complete(c) = &fin.state.kind {
                    FoundPath::Complete(*c)
                } else {
                    FoundPath::Path(FoldResult {
                        cache,
                        final_states
                    })
                }),
            })
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FinalState<R: ResultKind> {
    pub num_parents: usize,
    pub state: EndState<R>,
}
impl<R: ResultKind> PartialOrd for FinalState<R> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<R: ResultKind> Ord for FinalState<R> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.num_parents.cmp(&self.num_parents)
            .then_with(||
                self.state.is_complete().cmp(&other.state.is_complete())
                    .then_with(||
                        other.state.root.width().cmp(&self.state.root.width())
                    )
            )
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldResult<R: ResultKind> {
    pub cache: TraversalCache<R>,
    pub final_states: Vec<FinalState<R>>,
}
impl<R: ResultKind> FoldResult<R> {
    pub fn root_entry(&self) -> &PositionCache<R> {
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
    pub fn into_found_path(self) -> <R as ResultKind>::Found {
        //todo!("handle complete");
        let found = FoundPath::Path(self);
        <R as ResultKind>::Found::from(found)
    }
}