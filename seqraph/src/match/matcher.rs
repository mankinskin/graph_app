use crate::{
    r#match::*,
    Hypergraph,
};
use itertools::*;
use std::cmp::Ordering;

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
    pub(crate) fn searcher(&self) -> Searcher<'g, T, D> {
        Searcher::new(self.graph)
    }
    pub(crate) fn compare<'a>(
        &'g self,
        left: impl IntoPattern<Item = impl AsChild>,
        right: impl IntoPattern<Item = impl AsChild>,
    ) -> PatternMatchResult {
        let left: Pattern = left.into_pattern();
        let right: Pattern = right.into_pattern();
        if let Some((pos, eob)) = D::skip_equal_indices(left.iter(), right.iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(&lefti, &righti) => {
                    let left_context = D::front_context_normalized(left.as_pattern_view(), pos);
                    let right_context = D::front_context_normalized(right.as_pattern_view(), pos);
                    // Note: depending on sizes of left, right it may be differently efficient
                    // to search for children or parents, large patterns have less parents,
                    // small patterns have less children
                    // search larger in parents of smaller
                    let (rotate, sub, sub_context, sup, sup_context) =
                        // remember if sub and sup were switched
                        match lefti.width.cmp(&righti.width) {
                            // relatives can not have same sizes, mismatch occurred
                            Ordering::Equal => {
                                let left = D::split_end_normalized(left.as_pattern_view(), pos);
                                let right = D::split_end_normalized(right.as_pattern_view(), pos);
                                Err(PatternMismatchPath {
                                    path: vec![],
                                    remainder: EitherOrBoth::Both(left, right),
                                })
                            },
                            Ordering::Less => {
                                //println!("right super");
                                Ok((false, lefti, left_context, righti, right_context))
                            }
                            Ordering::Greater => {
                                //println!("left super");
                                Ok((true, righti, right_context, lefti, left_context))
                            }
                        }?;
                    match self.match_parent_with_child_in_context(sub, sub_context, sup) {
                        Ok(match_path) =>
                            match match_path.remainder {
                                MatchRemainder::Left(remainder)
                                    => self.compare(remainder, sup_context),
                                MatchRemainder::Right(remainder)
                                    => self.compare(sup_context, remainder),
                                MatchRemainder::None =>
                                    Ok(MatchPath {
                                        path: match_path.path,
                                        remainder: if sup_context.is_empty() {
                                            MatchRemainder::None
                                        } else {
                                            MatchRemainder::Right(sup_context)
                                        },
                                    }),
                            },
                        Err(MismatchPath {
                            path,
                            remainder,
                        }) => Err(PatternMismatchPath {
                            path,
                            remainder: match remainder {
                                MismatchRemainder::Query(query) => EitherOrBoth::Left(query),
                                MismatchRemainder::QueryAndChild(query, child_tail) => EitherOrBoth::Both(query, child_tail),
                            }
                        }),
                    }
                    .map_err(|mismatch_path| {
                        if rotate {
                            mismatch_path.flip_remainder()
                        } else {
                            mismatch_path
                        }
                    })
                    .map(|match_path| {
                        if rotate {
                            match_path.flip_remainder()
                        } else {
                            match_path
                        }
                    })
                }
                EitherOrBoth::Left(_) =>
                    Err(PatternMismatchPath {
                        path: vec![],
                        remainder: EitherOrBoth::Left(D::split_end_normalized(&left, pos)),
                    }),
                EitherOrBoth::Right(_) =>
                    Err(PatternMismatchPath {
                        path: vec![],
                        remainder: EitherOrBoth::Right(D::split_end_normalized(&right, pos)),
                    }),
            }
        } else {
            Ok(MatchPath::complete())
        }
    }
    fn match_parent_with_child_in_context<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        context: impl IntoPattern<Item = impl AsChild>,
        parent: impl Vertexed<'a, 'g>,
    ) -> Result<MatchPath, MismatchPath> {
        // try any parent, i.e. first
        let start_index = start.as_child();
        let parent_index = parent.as_child();
        let context = context.into_pattern();
        // breadth first traversal
        Bft::new((vec![], parent_index), |(path, parent)| {
            let vertex = self.expect_vertex_data(&parent);
            let child_patterns = vertex.get_children();
            child_patterns
                .into_iter()
                .map(|(&pid, pattern)| {
                    let &head = D::pattern_head(pattern).unwrap();
                    let mut path = path.clone();
                    path.push(ChildLocation::new(parent.clone(), pid, D::head_index(pattern)));
                    (path, head)
                })
                .collect_vec()
                .into_iter()
        })
        .find_map(|(_, (path, node))| {
            // find child starting with next_child
            (node == start_index).then(|| {
                // found start index at prefix of parent
                self.traceback_path(path, context.clone())
            })
        })
        // none found
        .unwrap_or_else(|| Err(MismatchPath {
            path: vec![],
            remainder: MismatchRemainder::Query(context),
        }))
    }
    /// returns: path to match/mismatch,
    pub(crate) fn traceback_path(
        &self,
        mut path: Vec<ChildLocation>,
        context: impl IntoPattern<Item = impl AsChild>,
    ) -> Result<MatchPath, MismatchPath> {
        let context = context.into_pattern();
        path.pop().map(|location| {
            // start index is some descendant of parent, with location
            let vertex = self.expect_vertex_data(location.parent);
            let child_patterns = vertex.get_children();
            let pattern = child_patterns.get(&location.pattern_id).unwrap();
            let child_tail = D::front_context_normalized(pattern, location.sub_index);
            self.compare(context.clone(), child_tail.clone())
                .or_else(|PatternMismatchPath {
                    path: inner_path,
                    remainder,
                }| {
                    let mut path = path.clone();
                    path.extend(inner_path);
                    match remainder {
                        EitherOrBoth::Left(remainder) =>
                            Ok(MatchPath {
                                path,
                                remainder: MatchRemainder::Left(remainder),
                            }),
                        EitherOrBoth::Right(remainder) =>
                            Ok(MatchPath {
                                path,
                                remainder: MatchRemainder::Right(remainder),
                            }),
                        EitherOrBoth::Both(left, right) =>
                            Err(MismatchPath {
                                path,
                                remainder: MismatchRemainder::QueryAndChild(left, right),
                            }),
                    }
                })
                .and_then(|match_path| match match_path.remainder {
                    MatchRemainder::Left(context) => {
                        // parent matches, continue with next parent and remaining context
                        self.traceback_path(path, context)
                    },
                    remainder@_ => {
                        // parent matches and context end
                        path.extend(match_path.path);
                        Ok(MatchPath {
                            path,
                            remainder,
                        })
                    },
                })
        })
        .unwrap_or_else(|| {
            // start index is parent? maybe disallowed
            Ok(MatchPath {
                path: vec![],
                remainder: MatchRemainder::Left(context),
            })
        })
    }
    // Outline:
    // matching two patterns of indices and
    // returning the remainder. Starting from left or right.
    // - skip equal indices
    // - once unequal, pick larger and smaller index
    // - search for larger in parents of smaller
    // - otherwise: try to find parent with best matching children
    //pub fn compare<
    //    A: IntoPattern<Item = impl AsChild>,
    //    B: IntoPattern<Item = impl AsChild>,
    //>(
    //    &self,
    //    a: A,
    //    b: B,
    //) -> PatternMatchResult {
    //    //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
    //    let a: Pattern = a.into_pattern();
    //    let b: Pattern = b.into_pattern();
    //    if let Some((pos, eob)) = D::skip_equal_indices(a.iter(), b.iter()) {
    //        match eob {
    //            // different elements on both sides
    //            EitherOrBoth::Both(&ai, &bi) => {
    //                let a_context = D::front_context_normalized(a.as_pattern_view(), pos);
    //                let b_context = D::front_context_normalized(b.as_pattern_view(), pos);
    //                // Note: depending on sizes of a, b it may be differently efficient
    //                // to search for children or parents, large patterns have less parents,
    //                // small patterns have less children
    //                // search larger in parents of smaller
    //                let (rotate, sub, sub_context, sup, sup_context) =
    //                    // remember if sub and sup were switched
    //                    match ai.width.cmp(&bi.width) {
    //                        // relatives can not have same sizes, mismatch occurred
    //                        Ordering::Equal => if pos > 0 {
    //                            return Ok(PatternMatch(
    //                                Some(D::split_end_normalized(a.as_pattern_view(), pos)),
    //                                Some(D::split_end_normalized(b.as_pattern_view(), pos)),
    //                            ));
    //                        } else {
    //                            Err(NoMatch::Mismatch)
    //                        },
    //                        Ordering::Less => {
    //                            //println!("right super");
    //                            Ok((false, ai, a_context, bi, b_context))
    //                        }
    //                        Ordering::Greater => {
    //                            //println!("left super");
    //                            Ok((true, bi, b_context, ai, a_context))
    //                        }
    //                    }?;
    //                self.find_unequal_matching_ancestor(sub, sub_context, sup)
    //                    .and_then(|parent_match| {
    //                        let rem = parent_match.remainder;
    //                        match parent_match.parent_range {
    //                            // continue if parent matches super completely
    //                            FoundRange::Complete => self.compare(rem.unwrap_or_default(), sup_context),
    //                            found_range => {
    //                                let post = D::get_remainder(found_range);
    //                                Ok(PatternMatch(
    //                                    rem,
    //                                    post.map(|post| {
    //                                        D::merge_remainder_with_context(post, sup_context.into_pattern())
    //                                    }),
    //                                ))
    //                            }
    //                        }
    //                    })
    //                    .map(|result| {
    //                        if rotate {
    //                            result.flip_remainder()
    //                        } else {
    //                            result
    //                        }
    //                    })
    //            }
    //            EitherOrBoth::Left(_) => {
    //                Ok(PatternMatch(Some(D::split_end_normalized(&a, pos)), None))
    //            }
    //            EitherOrBoth::Right(_) => {
    //                Ok(PatternMatch(None, Some(D::split_end_normalized(&b, pos))))
    //            }
    //        }
    //    } else {
    //        Ok(PatternMatch(None, None))
    //    }
    //}
    //pub(crate) fn find_unequal_matching_ancestor(
    //    &self,
    //    sub: impl Indexed,
    //    context: impl IntoPattern<Item = impl AsChild>,
    //    sup: Child,
    //) -> ParentMatchResult {
    //    let sub = sub.index();
    //    let vertex = self.expect_vertex_data(sub);
    //    if vertex.get_parents().is_empty() {
    //        return Err(NoMatch::NoParents);
    //    }
    //    // get parent where vertex is at relevant position (prefix or postfix)
    //    D::get_match_parent_to(&self.graph, vertex, sup)
    //        .and_then(|(pid, sub_index)| {
    //            // found vertex in sup at relevant position (prefix or postfix)
    //            //println!("sup found in parents");
    //            // compare context after vertex in parent
    //            let vert = self.expect_vertex_data(sup.index());
    //            let child_patterns = vert.get_children();
    //            let child_pattern = child_patterns
    //                .get(&pid)
    //                .expect("non existent pattern found as best match!");
    //            let (back_context, child_tail) = Self::split_child_context(child_pattern, sub_index);
    //            let (parent_match, _end_index) = self.child_context_at_offset_comparison(
    //                    context.as_pattern_view(),
    //                    back_context.as_pattern_view(),
    //                    child_tail.as_pattern_view(),
    //                    child_pattern.len()
    //                )
    //                .unwrap_or_else(|_| {
    //                    // if context doesn't match, at least current child matches
    //                    (ParentMatch {
    //                        parent_range: D::to_found_range(Some(child_tail.to_vec()), back_context),
    //                        remainder: Some(context.into_pattern()),
    //                    }, sub_index + 1)
    //                });
    //            Ok(parent_match)
    //        })
    //        .or_else(|_|
    //            // no relevant parent relation with sup
    //            // BFS
    //            self.searcher().find_matching_ancestor(
    //                vertex,
    //                context,
    //                Some(sup.width),
    //            )
    //            .and_then(
    //                |SearchFound {
    //                     location: PatternRangeLocation {
    //                         parent: parent_index,
    //                         ..
    //                    },
    //                     parent_match:
    //                         ParentMatch {
    //                             parent_range,
    //                             remainder,
    //                         },
    //                     ..
    //                 }|
    //                D::found_from_start(&parent_range)
    //                    .then(|| remainder.unwrap_or_default())
    //                    .ok_or(NoMatch::NoMatchingParent(sub))
    //                    .and_then(|new_context|
    //                        // sup is no direct parent, search upwards
    //                        //println!("matching available parents");
    //                        // search sup in parents
    //                        (parent_index == sup.index())
    //                        .then(|| ParentMatch {
    //                            parent_range: FoundRange::Complete,
    //                            remainder: (!new_context.is_empty()).then(|| new_context.clone()),
    //                        })
    //                        .map(Ok)
    //                        .unwrap_or_else(|| self.find_unequal_matching_ancestor(parent_index, new_context, sup))
    //                    )
    //            )
    //        )
    //}
}
