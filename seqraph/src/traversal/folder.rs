use crate::*;
use super::*;

pub type Folder<T, D, Q, R, Ty>
    = <Ty as DirectedTraversalPolicy<T, D, Q, R>>::Trav;

pub trait FolderQ<
    T: Tokenize,
    D: MatchDirection,
    Q: QueryPath,
    S: DirectedTraversalPolicy<T, D, Q, R>,
    R: ResultKind,
> {
    type Query: QueryPath;
}

impl<
    T: Tokenize,
    D: MatchDirection,
    Q: QueryPath,
    R: ResultKind,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Ty>,
    Ty: TraversalFolder<T, D, Q, S, R>,
> FolderQ<T, D, Q, S, R> for Ty {
    type Query = Q;
}

pub type FolderQuery<T, D, Q, R, S>
    = <Folder<T, D, Q, R, S> as FolderQ<T, D, Q, S, R>>::Query;

pub type FolderPathPair<T, D, Q, R, S>
    = PathPair<<R as ResultKind>::Advanced, FolderQuery<T, D, Q, R, S>>;

struct FoldResult<R: ResultKind, Q: QueryPath> {
    cache: TraversalCache<R, Q>,
    end_states: Vec<(CacheKey, EndState<R, Q>)>,
}
pub trait TraversalFolder<
    T: Tokenize,
    D: MatchDirection,
    Q: QueryPath,
    S: DirectedTraversalPolicy<T, D, Q, R, Trav=Self>,
    R: ResultKind,
>: Sized + Traversable<T> {
    //type Break;
    //type Continue;
    //type Result = ControlFlow<Self::Break, Self::Continue>;
    type NodeCollection: NodeCollection<R, Q>;

    //fn map_state(
    //    &self,
    //    acc: ControlFlow<Self::Break, Self::Continue>,
    //    node: TraversalState<R, Q>
    //) -> ControlFlow<Self::Break, Self::Continue>;

    fn fold_query<P: IntoPattern>(
        &self,
        query: P,
    ) -> Result<(FoldResult<R, Q>, Q), (NoMatch, Q)> {
        let query_path = Q::new_directed::<D, _>(query.borrow())
            .map_err(|(err, q)| (err, q))?;
        let index = query_path.path_child(self);
        let start = StartState::new(index, query_path);

        let mut states = OrderedTraverser::<_, _, _, _, _, _, Self::NodeCollection>::new(self, start);
        let mut cache = TraversalCache::new();
        let mut end_states = vec![];

        while let Some((depth, state)) = states.next() {
            match cache.add_state(&state) {
                Ok(key) =>
                    match states.next_states(key, state) {
                        NextStates::End(prev, state) => {
                            if let Some(root_key) = state.waiting_root_key() {
                                states.extend(
                                    cache.continue_waiting(&root_key)
                                );
                            }
                            end_states.push((prev, state));
                        },
                        next_states => {
                            states.extend(
                                next_states.into_states()
                                    .into_iter()
                                    .map(|node| (depth + 1, node))
                            );
                        },
                    },
                Err(key) =>
                    cache.get_entry_mut(&key)
                        .unwrap()
                        .add_waiting(depth, state)
            }
        }
        Ok((FoldResult {
            cache,
            end_states
        }, query_path))
    }
}