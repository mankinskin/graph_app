use crate::*;

pub mod exit;
pub mod from_advanced;
pub mod into_advanced;
pub mod into_primer;

pub use exit::*;
pub use from_advanced::*;
pub use into_advanced::*;
pub use into_primer::*;

//pub trait AdvanceState {
//    type Next;
//    fn advance<
//        Trav: Traversable,
//    >(
//        self,
//        trav: &Trav,
//        cache: &mut TraversalCache,
//    ) -> Self::Next;
//}
//impl AdvanceState for ParentState {
//    type Next = Option<ChildState>;
//    fn advance<
//        Trav: Traversable,
//    >(
//        self,
//        trav: &Trav,
//        cache: &mut TraversalCache,
//    ) -> Self::Next {
//
//        if self.entry.advance_leaf(trav).is_continue() {
//            Some(ChildState {
//                root: CacheKey {
//                    index: self.entry.index(),
//                    token_pos: 0,
//                },
//                location: self.entry,
//                mode: PathPairMode::GraphMajor
//            })
//        } else {
//            None
//        }
//    }
//}
//impl AdvanceState for ChildState {
//    type Next = Vec<ChildState>;
//    fn advance<
//        Trav: Traversable,
//    >(
//        self,
//        trav: &Trav,
//        cache: &mut TraversalCache,
//    ) -> Self::Next {
//        self.location.advance(trav, cache)
//            .into_iter()
//            .map(|loc|
//                ChildState {
//                    location: loc,
//                    root: self.root,
//                    mode: self.mode,
//                }
//            )
//            .collect()
//    }
//}
//impl AdvanceState for ChildLocation {
//    type Next = Vec<ChildLocation>;
//    fn advance<
//        Trav: Traversable,
//    >(
//        self,
//        trav: &Trav,
//        cache: &mut TraversalCache,
//    ) -> Self::Next {
//        if self.advance_leaf(trav).is_continue() {
//            vec![self]
//        } else {
//            let back_edges = cache.get_entry(self.location.target_key())
//                .unwrap()
//                .back_edges;
//            back_edges.iter()
//                .filter_map(|(k, e)| if let CacheEdge::TopDown(loc) = e {
//                    Some(k.index().to_child_location(loc))
//                } else {
//                    None
//                })
//                .map(|loc|
//                    loc.advance(trav, cache)
//                )
//                .flatten()
//                .collect_vec()
//        }
//    }
//}
//impl AdvanceState for QueryState {
//    fn advance<
//        Trav: Traversable,
//    >(
//        self,
//        trav: &Trav,
//        cache: &mut TraversalCache,
//    ) -> Self::Next {
//        let mut query = CachedQuery {
//            state: self,
//            cache,
//        };
//        if query.advance_leaf(trav).is_continue() {
//            query.state
//        } else {
//            match query.state {
//                QueryState::TopDown(prev, entry) => {
//                    let back_edges = cache.get_entry(self.prev)
//                        .unwrap()
//                        .back_edges;
//                    back_edges.iter()
//                        .filter_map(|(k, e)| if let CacheEdge::TopDownQuery(loc) = e {
//                            Some(k.index().to_child_location(loc))
//                        } else {
//                            None
//                        })
//                        .map(|loc|
//                            loc.advance(trav, cache)
//                        )
//                        .flatten()
//                        .collect_vec()
//                }
//                QueryState::RootExit(exit) => {
//                }
//            }
//        }
//    }
//}


pub trait Advance:
    PathPop
    + PathAppend
    + AdvanceRootPos<End>
{
    fn advance<
        Trav: Traversable,
    >(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        if let Some(location) = std::iter::from_fn(|| 
            self.path_pop()
        ).find_map(|mut location| {
            location.advance_leaf(&graph).is_continue()
                .then(|| location)
        }) {
            self.path_append(location);
            ControlFlow::CONTINUE
        } else {
            self.advance_root_pos(trav)
        }
    }
}
impl<T: 
    AdvanceRootPos<End>
    + PathPop
    + PathAppend
    + Sized
> Advance for T {
}
pub trait AdvanceLeaf {
    fn advance_leaf<
        Trav: Traversable,
    >(&mut self,
        trav: &Trav,
    ) -> ControlFlow<()>;
}
impl AdvanceLeaf for ChildLocation {
    fn advance_leaf<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&*self);
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), self.sub_index) {
            self.sub_index = next;
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}
impl AdvanceLeaf for CachedQuery<'_> {
    fn advance_leaf<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = if let Some(loc) = self.state.end.path_child_location() {
            graph.expect_pattern_at(loc)
        } else {
            &self.cache.query_root
        };
        let exit = self.state.end.leaf_child_pos_mut();
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), *exit) {
            *exit = next;
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}
impl AdvanceRootPos<End> for CachedQuery<'_> {
    fn advance_root_pos<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = &self.cache.query_root;
        let exit = self.state.end.root_child_pos_mut();
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), *exit) {
            *exit = next;
            ControlFlow::CONTINUE
        } else {
            ControlFlow::BREAK
        }
    }
}
//impl PathAppend for CachedQuery<'_> {
//    fn path_append(
//            &mut self,
//            parent_entry: ChildLocation
//    ) {
//        match self.state {
//            QueryState::TopDown(prev, entry) => {
//            },
//            QueryState::RootExit(_) => {
//            },
//        }
//    }
//}
//impl PathPop for CachedQuery<'_> {
//    fn path_pop(&mut self) -> Option<ChildLocation> {
//        match self.state {
//            QueryState::TopDown(prev, entry) => {
//            }
//            QueryState::RootExit(_) => None,
//        }
//    }
//}
//pub trait AdvanceWidth {
//    fn advance_width(&mut self, width: usize);
//}
//impl <T: WideMut> AdvanceWidth for T {
//    fn advance_width(&mut self, width: usize) {
//        *self.width_mut() += width;
//    }
//}
//
//pub trait AddMatchWidth: AdvanceWidth + LeafChild<End> {
//    fn add_match_width<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&mut self, trav: &Trav) {
//        let leaf = self.leaf_child(trav);
//        self.advance_width(leaf.width);
//    }
//}
//impl<T: AdvanceWidth + LeafChild<End>> AddMatchWidth for T {
//}
//impl AdvanceWidth for QueryRangePath {
//    fn advance_width(&mut self, _width: usize) {
//    }
//}