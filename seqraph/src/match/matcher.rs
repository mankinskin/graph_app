use crate::{
    r#match::*,
    Child,
    ChildPatterns,
    Hypergraph,
    Indexed,
    PatternId,
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
    // Outline:
    // matching two patterns of indices and
    // returning the remainder. Starting from left or right.
    // - skip equal indices
    // - once unequal, pick larger and smaller index
    // - search for larger in parents of smaller
    // - otherwise: try to find parent with best matching children
    pub fn compare<
        A: IntoPattern<Item = impl AsChild>,
        B: IntoPattern<Item = impl AsChild>,
    >(
        &self,
        a: A,
        b: B,
    ) -> PatternMatchResult {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        let a: Pattern = a.into_pattern();
        let b: Pattern = b.into_pattern();
        if let Some((pos, eob)) = D::skip_equal_indices(a.iter(), b.iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(&ai, &bi) => {
                    let a_context = D::front_context_normalized(a.as_pattern_view(), pos);
                    let b_context = D::front_context_normalized(b.as_pattern_view(), pos);
                    // Note: depending on sizes of a, b it may be differently efficient
                    // to search for children or parents, large patterns have less parents,
                    // small patterns have less children
                    // search larger in parents of smaller
                    let (rotate, sub, sub_context, sup, sup_context) =
                        // remember if sub and sup were switched
                        match ai.width.cmp(&bi.width) {
                            // relatives can not have same sizes, mismatch occurred
                            Ordering::Equal => if pos > 0 {
                                return Ok(PatternMatch(
                                    Some(D::split_end_normalized(a.as_pattern_view(), pos)),
                                    Some(D::split_end_normalized(b.as_pattern_view(), pos)),
                                ));
                            } else {
                                Err(NoMatch::Mismatch)    
                            },
                            Ordering::Less => {
                                //println!("right super");
                                Ok((false, ai, a_context, bi, b_context))
                            }
                            Ordering::Greater => {
                                //println!("left super");
                                Ok((true, bi, b_context, ai, a_context))
                            }
                        }?;
                    self.find_unequal_matching_ancestor(sub, sub_context, sup)
                        .and_then(|parent_match| {
                            let rem = parent_match.remainder;
                            match parent_match.parent_range {
                                // continue if parent matches super completely
                                FoundRange::Complete => self.compare(rem.unwrap_or_default(), sup_context),
                                found_range => {
                                    let post = D::get_remainder(found_range);
                                    Ok(PatternMatch(
                                        rem,
                                        post.map(|post| {
                                            D::merge_remainder_with_context(post, sup_context.into_pattern())
                                        }),
                                    ))
                                }
                            }
                        })
                        .map(|result| {
                            if rotate {
                                result.flip_remainder()
                            } else {
                                result
                            }
                        })
                }
                EitherOrBoth::Left(_) => {
                    Ok(PatternMatch(Some(D::split_end_normalized(&a, pos)), None))
                }
                EitherOrBoth::Right(_) => {
                    Ok(PatternMatch(None, Some(D::split_end_normalized(&b, pos))))
                }
            }
        } else {
            Ok(PatternMatch(None, None))
        }
    }
    pub(crate) fn find_unequal_matching_ancestor(
        &self,
        sub: impl Indexed,
        context: impl IntoPattern<Item = impl AsChild>,
        sup: Child,
    ) -> ParentMatchResult {
        let sub_index = sub.index();
        let vertex = self.expect_vertex_data(sub);
        if vertex.get_parents().is_empty() {
            return Err(NoMatch::NoParents);
        }
        // get parent where vertex is at relevant position (prefix or postfix)
        D::get_match_parent_to(&self.graph, vertex, sup)
            .and_then(|(pid, sub_index)|
                // found vertex in sup at relevant position (prefix or postfix)
                //println!("sup found in parents");
                // compare context after vertex in parent
                Ok(self.match_context_with_parent_child(
                    context.as_pattern_view(),
                    sup,
                    pid,
                    sub_index,
                ))
            )
            .or_else(|_|
                // no relevant parent relation with sup
                self.searcher().find_largest_matching_ancestor(
                    vertex,
                    context,
                    Some(sup.width),
                )
                .and_then(
                    |SearchFound {
                         location: PatternRangeLocation {
                             parent: parent_index,
                             ..
                        },
                         parent_match:
                             ParentMatch {
                                 parent_range,
                                 remainder,
                             },
                         ..
                     }|
                    D::found_at_start(&parent_range)
                        .then(|| remainder.unwrap_or_default())
                        .ok_or(NoMatch::NoMatchingParent(sub_index))
                        .and_then(|new_context|
                            // sup is no direct parent, search upwards
                            //println!("matching available parents");
                            // search sup in parents
                            (parent_index == sup.index())
                            .then(|| ParentMatch {
                                parent_range: FoundRange::Complete,
                                remainder: (!new_context.is_empty()).then(|| new_context.clone()),
                            })
                            .map(Ok)
                            .unwrap_or_else(|| self.find_unequal_matching_ancestor(parent_index, new_context, sup))
                        )
                )
            )
    }
    /// match context against child context in parent.
    pub(crate) fn match_context_with_parent_child(
        &'g self,
        context: impl IntoPattern<Item = impl AsChild>,
        parent: Child,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> ParentMatch {
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent.index());
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub index
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        let child_pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, child_tail) = Self::split_child_context(child_pattern, sub_index);
        let (parent_match, _end_index) = self.child_context_at_offset_comparison(
                context.as_pattern_view(),
                back_context.as_pattern_view(),
                child_tail.as_pattern_view(),
                child_pattern.len()
            )
            .unwrap_or_else(|_| {
                // if context doesn't match, at least current child matches
                (ParentMatch {
                    parent_range: D::to_found_range(Some(child_tail.to_vec()), back_context),
                    remainder: Some(context.into_pattern()),
                }, sub_index + 1)
            });
        parent_match
    }
    pub(crate) fn split_child_context(
        child_pattern: &Pattern,
        sub_index: usize,
    ) -> (Pattern, Pattern) {
        let (back_context, rem) = D::directed_pattern_split(child_pattern, sub_index);
        let child_tail = D::pattern_tail(&rem[..]);
        (back_context, child_tail.to_vec())
    }
    /// comparison on child pattern and context
    /// returns ParentMatch and end index of match
    /// matches only when context matches at least one index
    pub(crate) fn compare_child_context_at_offset(
        &'g self,
        child_patterns: &'g ChildPatterns,
        context: impl IntoPattern<Item = impl AsChild>,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> Result<(ParentMatch, usize), NoMatch> {
        let child_pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, child_tail) = Self::split_child_context(child_pattern, sub_index);
        // match search context with child tail
        // back context is from child pattern
        self.child_context_at_offset_comparison(context, back_context, child_tail, child_pattern.len())
        // returns result of matching sub with parent's children
    }
    pub(crate) fn child_context_at_offset_comparison(
        &self,
        context: impl IntoPattern<Item = impl AsChild>,
        back_context: impl IntoPattern<Item = impl AsChild>,
        child_tail: impl IntoPattern<Item = impl AsChild>,
        child_pattern_len: usize,
    ) -> Result<(ParentMatch, usize), NoMatch> {
        // match search context with child tail
        // back context is from child pattern
        self.compare(context.as_pattern_view(), child_tail).map(|pm| {
            let post_len = pm.1.as_ref().map(|p| p.len()).unwrap_or_default();
            let end = child_pattern_len-post_len;
            (ParentMatch {
                parent_range: D::to_found_range(pm.1, back_context.into_pattern()),
                remainder: pm.0,
            }, end)
        })
    }
    //#[allow(unused)]
    ///// match sub index and context with sup index with max width
    //fn match_sub_and_context_with_index(
    //    &self,
    //    sub: impl Indexed,
    //    context: impl IntoPattern<Item = impl AsChild + Clone>,
    //    sup: Child,
    //) -> ParentMatchResult {
    //    Self::try_exact_match(sub.index(), context.as_pattern_view(), sup)
    //        .map(Ok)
    //        .unwrap_or_else(|| {
    //            self.match_sub_vertex_and_context_with_index(
    //                self.expect_vertex_data(sub),
    //                context,
    //                sup,
    //            )
    //        })
    //}
}
