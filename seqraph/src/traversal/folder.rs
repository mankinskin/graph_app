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
                    let state = next.inner;
                    if state.matched {
                        // stop other paths not with this root
                        if !matches!(state.kind, EndKind::Complete(_)) && cache.expect(&state.root_key()).num_bu_edges() < 2 {
                            //states.prune_not_below(state.root_key());
                        }
                        if let Some(root_key) = state.waiting_root_key() {
                            // this must happen before simplification
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
        Ok(if end_states.is_empty() {
            TraversalResult {
                query: query.to_rooted(query_pattern),
                result: FoldResult::Complete(index)
            }
        } else {

            // todo: find single root and end state
            let final_states = end_states.iter()
                .map(|state|
                    FinalState {
                        num_parents: cache
                            .get(&state.root_key())
                            .unwrap()
                            .num_parents(),
                        state,
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
            let fin = final_states.first().unwrap();
            let query = fin.state.query.clone();
            let found_path = if let EndKind::Complete(c) = &fin.state.kind {
                    FoldResult::Complete(*c)
                } else {
                    //cache.trace_subgraph(&end_states);
                    FoldResult::Incomplete(FoldState {
                        cache,
                        end_states,
                        start: start_key,
                    })
                };
            TraversalResult {
                query: query.to_rooted(query_pattern),
                result: found_path,
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
pub struct FinalState<'a> {
    pub num_parents: usize,
    pub state: &'a EndState,
}
impl PartialOrd for FinalState<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FinalState<'_> {
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
pub struct FoldState {
    pub cache: TraversalCache,
    pub end_states: Vec<EndState>,
    pub start: CacheKey,
}
impl FoldState {
    pub fn root_entry(&self) -> &VertexCache {
        self.cache.entries.get(&self.root_index().index()).unwrap()
    }
    pub fn root_index(&self) -> Child {
        // assert same root
        let root = self.end_states.first().unwrap().root_key().index;
        assert!(
            self.end_states
                .iter()
                .skip(1)
                .all(|state|
                    state.root_key().index == root
                )
        );
        root
    }
    pub fn into_fold_result(self) -> FoldResult {
        FoldResult::Incomplete(self)
    }
    pub fn roots(&self) -> Vec<CacheKey> {
        self.end_states.iter().map(|s| s.root_key()).collect()
    }
    pub fn leaves(&self, root: &CacheKey) -> Vec<CacheKey> {
        self.end_states.iter()
            .filter(|s| s.root_key() == *root)
            .map(|s| s.target_key())
            .collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FoldResult {
    Complete(Child),
    Incomplete(FoldState),
}

// get bottom up edge iterators
//  - use back edges for late path directly
//  - trace back edges for early path to gather bottom up edges
//    - build new cache for this or store forward edges directly in search
// edge: child location, position
// tabularize all splits bottom up
// table: index, position -> split
// breadth first bottom up traversal , merging splits
// - start walking edges up from leaf nodes
// - each edge has location in parent and position
//    - each edge defines a split in parent at location, possibly merged with nested splits from below path
// - combine splits into a pair of halves for each position
//    - each position needs a single pair of halves, built with respect to other positions
// - continue walk up to parents, write split halves to table for each position
//    - use table to pass finished splits upwards
// - combine split context and all positions into pairs of halves for each position
// - at root, use both edges and splits to build inner and outer pieces
