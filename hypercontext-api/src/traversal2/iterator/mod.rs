pub mod cache;
pub mod states;
//
// Traversal Iterator Spec
//
// Traversable, Pattern -> RangePath -> TraversalIterator -> TraversalResult
//                          search
//                                       TraversalCache
//                                       TraversalState
//                                       NextStates
//                                       TraversalContext
//                                       TraversalStateContext
//                                                             FoldFound
//