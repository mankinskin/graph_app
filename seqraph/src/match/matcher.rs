use crate::{
    r#match::*,
    Hypergraph,
};
use itertools::*;

#[derive(Clone, Debug)]
pub struct Matcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'g, T: Tokenize, D: MatchDirection> std::ops::Deref for Matcher<'g, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Matcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    /// match a query with the context of a child at a path
    pub(crate) fn match_path_in_context(
        &self,
        mut path: Vec<ChildLocation>,
        query: impl IntoPattern<Item = impl AsChild>,
    ) -> Result<GrownPath, MismatchPath> {
        let query = query.into_pattern();
        path.pop().map(|mut location| {
            // start index is some descendant of parent, with location
            let vertex = self.expect_vertex_data(location.parent);
            let child_patterns = vertex.get_children();
            let pattern = child_patterns.get(&location.pattern_id).unwrap();
            let child_tail = D::front_context(pattern, location.sub_index);
            if let Some((skipped, eob)) = D::skip_equal_indices(child_tail.iter(), query.iter()) {
                match eob {
                    // different elements on both sides
                    EitherOrBoth::Both(&child_next, &query_next) => {
                        // select larger and smaller
                            if child_next.width <= query_next.width {
                                // relatives can not have same sizes or
                                // next query is larger than next child, mismatch
                                location.sub_index += skipped;
                                path.push(location);
                                Err(MismatchPath {
                                    path,
                                    child: D::split_end_normalized(child_tail.as_pattern_view(), skipped),
                                    query: D::split_end_normalized(query.as_pattern_view(), skipped),
                                })
                            } else {
                                // next child is larger than next query, search next query in prefix of next child
                                let query_context = D::front_context_normalized(query.as_pattern_view(), skipped);
                                //let child_context = D::front_context_normalized(child_tail.as_pattern_view(), skipped);
                                let child_next = child_next.as_child();
                                let query_next = query_next.as_child();
                                // breadth first find query_next in child_next
                                location.sub_index += skipped;
                                let mut loc = location.clone();
                                loc.sub_index += 1;
                                Bft::new((vec![loc], child_next), |(path, parent)| {
                                    let vertex = self.expect_vertex_data(&parent);
                                    let child_patterns = vertex.get_children();
                                    child_patterns
                                        .into_iter()
                                        .filter_map(|(&pid, pattern)| {
                                            let &head = D::pattern_head(pattern).unwrap();
                                            if head.width() >= query_next.width() {
                                                let mut path = path.clone();
                                                let next_index = D::head_index(pattern);
                                                path.push(ChildLocation::new(parent.clone(), pid, next_index));
                                                Some((path, head))
                                            } else {
                                                None
                                            }
                                        })
                                        .collect_vec()
                                        .into_iter()
                                })
                                // find child starting with next_child
                                .find_map(|(_, (path, node))|
                                    (node == query_next).then(||
                                        Ok(path)
                                    )
                                )
                                // none found
                                .unwrap_or_else(|| {
                                    path.push(location);
                                    Err(MismatchPath {
                                        // todo add path
                                        path: path.clone(),
                                        child: D::split_end_normalized(child_tail.as_pattern_view(), skipped),
                                        query: D::split_end_normalized(query.as_pattern_view(), skipped),
                                    })
                                })
                                // found path to query_next in child_next
                                .and_then(|inner_path| {
                                    path.extend(inner_path);
                                    self.match_path_in_context(path, query_context)
                                })
                            }
                    }
                    EitherOrBoth::Right(_) => {
                        // child tail ended, so the remaining path points to a matching index
                        let query_context = D::split_end_normalized(&query, skipped);
                        self.match_path_in_context(path, query_context)
                    },
                    EitherOrBoth::Left(_) => {
                        // query ended
                        location.sub_index += skipped;
                        path.push(location);
                        Ok(GrownPath {
                            path,
                            remainder: GrowthRemainder::Child(D::split_end_normalized(&child_tail, skipped)),
                        })
                    },
                }
            } else {
                self.match_path_in_context(path, vec![] as Pattern)
            }
        })
        .unwrap_or_else(|| {
            Ok(GrownPath {
                path: vec![],
                remainder: if query.is_empty() {
                    GrowthRemainder::None
                } else {
                    GrowthRemainder::Query(query)
                },
            })
        })
    }
    // match a query with the context of a child at a path
    //pub(crate) fn match_path_in_context(
    //    &self,
    //    mut path: Vec<ChildLocation>,
    //    query: impl IntoPattern<Item = impl AsChild>,
    //) -> Result<GrownPath, MismatchPath> {
    //    let query = query.into_pattern();
    //    path.pop().map(|location| {
    //        // start index is some descendant of parent, with location
    //        let vertex = self.expect_vertex_data(location.parent);
    //        let child_patterns = vertex.get_children();
    //        let pattern = child_patterns.get(&location.pattern_id).unwrap();
    //        let child_tail = D::front_context(pattern, location.sub_index);
    //        self.compare(child_tail.clone(), query.clone())
    //            .or_else(|MismatchPath {
    //                path: inner_path,
    //                left,
    //                right,
    //            }| {
    //                let mut path = path.clone();
    //                path.push(ChildLocation::new(
    //                    location.parent,
    //                    location.pattern_id,
    //                    D::tail_index(pattern, &left),
    //                ));
    //                path.extend(inner_path);
    //                Err(MismatchPath {
    //                    path,
    //                    left,
    //                    right,
    //                })
    //            })
    //            .and_then(|match_path| {
    //                match match_path.remainder {
    //                    MatchRemainder::Right(query) => {
    //                        // parent matches, continue with next parent and remaining context
    //                        self.match_path_in_context(path, query)
    //                    },
    //                    MatchRemainder::Left(child_tail) => {
    //                        // parent matches and context end
    //                        //path.push(ChildLocation::new(
    //                        //    location.parent,
    //                        //    location.pattern_id,
    //                        //    D::tail_index(pattern, &child_tail),
    //                        //));
    //                        path.extend(match_path.path);
    //                        Ok(GrownPath {
    //                            path,
    //                            remainder: GrowRemainder::Child(child_tail),
    //                        })
    //                    },
    //                    MatchRemainder::None => {
    //                        // parent matches and context end
    //                        path.extend(match_path.path);
    //                        Ok(GrownPath {
    //                            path,
    //                            remainder: GrowRemainder::None,
    //                        })
    //                    },
    //                }
    //            })
    //    })
    //    .unwrap_or_else(|| {
    //        Ok(GrownPath {
    //            path: vec![],
    //            remainder: GrowRemainder::Context(query),
    //        })
    //    })
    //}
    //pub(crate) fn compare<'a>(
    //    &'g self,
    //    left: impl IntoPattern<Item = impl AsChild>,
    //    right: impl IntoPattern<Item = impl AsChild>,
    //) -> MatchResult {
    //    let left: Pattern = left.into_pattern();
    //    let right: Pattern = right.into_pattern();
    //    if let Some((pos, eob)) = D::skip_equal_indices(left.iter(), right.iter()) {
    //        match eob {
    //            // different elements on both sides
    //            EitherOrBoth::Both(&lefti, &righti) => {
    //                let left_context = D::front_context_normalized(left.as_pattern_view(), pos);
    //                let right_context = D::front_context_normalized(right.as_pattern_view(), pos);
    //                let (rotate, sub, sub_context, sup, sup_context) =
    //                    // remember if sub and sup were switched
    //                    match lefti.width.cmp(&righti.width) {
    //                        // relatives can not have same sizes, mismatch occurred
    //                        Ordering::Equal => Err(MismatchPath {
    //                                path: vec![],
    //                                left: D::split_end_normalized(left.as_pattern_view(), pos),
    //                                right: D::split_end_normalized(right.as_pattern_view(), pos),
    //                            }),
    //                        Ordering::Less => {
    //                            //println!("right super");
    //                            Ok((false, lefti, left_context, righti, right_context))
    //                        }
    //                        Ordering::Greater => {
    //                            //println!("left super");
    //                            Ok((true, righti, right_context, lefti, left_context))
    //                        }
    //                    }?;
    //                    let start_index = sub.as_child();
    //                    let sub_context = sub_context.into_pattern();
    //                    let parent_index = sup.as_child();
    //                    // breadth first traversal
    //                    Bft::new((vec![], parent_index), |(path, parent)| {
    //                        let vertex = self.expect_vertex_data(&parent);
    //                        let child_patterns = vertex.get_children();
    //                        child_patterns
    //                            .into_iter()
    //                            .filter_map(|(&pid, pattern)| {
    //                                let &head = D::pattern_head(pattern).unwrap();
    //                                if head.width() >= start_index.width() {
    //                                    let mut path = path.clone();
    //                                    let next_index = D::head_index(pattern);
    //                                    path.push(ChildLocation::new(parent.clone(), pid, next_index));
    //                                    Some((path, head))
    //                                } else {
    //                                    None
    //                                }
    //                            })
    //                            .collect_vec()
    //                            .into_iter()
    //                    })
    //                    .find_map(|(_, (path, node))| {
    //                        // find child starting with next_child
    //                        (node == start_index).then(|| {
    //                            // found start index at prefix of parent
    //                            //let sub_index = &mut path.last_mut().unwrap().sub_index;
    //                            //*sub_index = D::index_next(*sub_index).unwrap();
    //                            self.match_path_in_context(path, sub_context.clone())
    //                        })
    //                    })
    //                    // none found
    //                    .unwrap_or_else(|| Err(MismatchPath {
    //                        // todo add path
    //                        path: vec![],
    //                        left: D::split_end_normalized(left.as_pattern_view(), pos),
    //                        right: D::split_end_normalized(right.as_pattern_view(), pos),
    //                    }))
    //                    .and_then(|grown_path| {
    //                        match grown_path.remainder {
    //                            GrowRemainder::Context(sub_remainder)
    //                                => self.compare(sub_remainder, sup_context),
    //                            GrowRemainder::Child(mut sup_remainder) => {
    //                                sup_remainder = D::merge_remainder_with_context(sup_remainder, sup_context);
    //                                Ok(MatchPath {
    //                                    path: grown_path.path,
    //                                    remainder: MatchRemainder::Right(sup_remainder),
    //                                })
    //                            },
    //                            GrowRemainder::None =>
    //                                Ok(MatchPath {
    //                                    path: grown_path.path,
    //                                    remainder: if sup_context.is_empty() {
    //                                        MatchRemainder::None
    //                                    } else {
    //                                        MatchRemainder::Right(sup_context)
    //                                    },
    //                                }),
    //                        }
    //                        .map_err(|mismatch_path|
    //                            if rotate {
    //                                mismatch_path.flip_remainder()
    //                            } else {
    //                                mismatch_path
    //                            }
    //                        )
    //                    })
    //                    .map(|match_path|
    //                        if rotate {
    //                            match_path.flip_remainder()
    //                        } else {
    //                            match_path
    //                        }
    //                    )
    //            }
    //            EitherOrBoth::Left(_) =>
    //                Ok(MatchPath {
    //                    path: vec![],
    //                    remainder: MatchRemainder::Left(D::split_end_normalized(&left, pos)),
    //                }),
    //            EitherOrBoth::Right(_) =>
    //                Ok(MatchPath {
    //                    path: vec![],
    //                    remainder: MatchRemainder::Right(D::split_end_normalized(&right, pos)),
    //                }),
    //        }
    //    } else {
    //        Ok(MatchPath::complete())
    //    }
    //}
}
