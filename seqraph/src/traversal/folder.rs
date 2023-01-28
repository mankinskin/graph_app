use crate::*;
use super::*;

pub type Folder<T, D, R, Ty>
    = <Ty as DirectedTraversalPolicy<T, D, R>>::Trav;

pub type FolderPathPair<R>
    = PathPair<<R as ResultKind>::Advanced, <R as ResultKind>::Query>;

pub trait TraversalFolder<
    T: Tokenize,
    D: MatchDirection,
    S: DirectedTraversalPolicy<T, D, R, Trav=Self>,
    R: ResultKind,
>: Sized + Traversable<T> {
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
        let query_path = R::Query::new_directed::<D, _>(query.borrow())
            .map_err(|(err, q)| (err, q))?;
        let index = query_path.leaf_child(self);

        let start = StartState::new(index, query_path.clone());
        let mut states = OrderedTraverser::<_, _, _, _, _, Self::NodeCollection>::new(self);
        let (start_key, mut cache) = TraversalCache::new(&start);
        states.extend(
            states.query_start(start_key, start)
                .into_states()
                .into_iter()
                .map(|n| (1, n))
        );
        let mut end_states = vec![];

        while let Some((depth, next_states)) = states.next_states(&mut cache) {
            match &next_states {
                NextStates::End(prev, matched, state) => {
                    if *matched {
                        // stop other paths not with this root
                        states.prune_not_below(state.root_key());
                    } else {
                        // stop other paths with this root
                        states.prune_below(state.root_key());
                    }
                    if let Some(root_key) = state.waiting_root_key() {
                        states.extend(
                            cache.continue_waiting(&root_key)
                        );
                    }
                    end_states.push((*prev, state.clone()));
                },
                _ => {},
            }
            states.extend(
                next_states.into_states()
                    .into_iter()
                    .map(|nstate| (depth, nstate))
            );
        }
        let result = FoldResult {
            cache,
            end_states
        };
        Ok(TraversalResult {
            path: result.into_found_path(),
            query: query_path,
        })
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoldResult<R: ResultKind> {
    pub cache: TraversalCache<R>,
    pub end_states: Vec<(CacheKey, EndState<R>)>,
}
impl<R: ResultKind> FoldResult<R> {
    pub fn root_entry(&self) -> &PositionCache<R> {
        self.cache.entries.get(&self.root_index().index()).unwrap()
    }
    pub fn root_index(&self) -> Child {
        // assert same root
        let root = self.end_states.first().unwrap().1.root_key().index;
        assert!(
            self.end_states
                .iter()
                .skip(1)
                .all(|s|
                    s.1.root_key().index == root
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