use crate::{
    r#match::*,
    Child,
    ChildPatterns,
    Hypergraph,
    Indexed,
    Parent,
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
        let sub_index = *sub.index();
        let vertex = self.expect_vertex_data(sub);
        if vertex.get_parents().is_empty() {
            return Err(NoMatch::NoParents);
        }
        // get parent where vertex is at relevant position (prefix or postfix)
        D::get_match_parent_to(vertex, sup)
            .and_then(|parent|
                // found vertex in sup at relevant position
                //println!("sup found in parents");
                // compare context after vertex in parent
                self.match_context_with_parent_children(
                    context.as_pattern_view(),
                    sup,
                    parent,
                )
                .map(|(parent_match, _)| parent_match)
            )
            .or_else(|_|
                self.searcher().find_largest_matching_ancestor(
                    vertex,
                    context,
                    Some(sup.width),
                )
                .and_then(
                    |SearchFound {
                         location: PatternLocation {
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
                            (parent_index == *sup.index())
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
    pub(crate) fn match_context_with_parent_children(
        &'g self,
        context: impl IntoPattern<Item = impl AsChild>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Result<(ParentMatch, PatternLocation), NoMatch> {
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent_index);
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub index
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        D::filter_parent_pattern_indices(parent, child_patterns)
            .into_iter()
            .find_or_first(|(pattern_index, sub_index)|
                D::compare_next_index_in_child_pattern(
                    child_patterns,
                    context.as_pattern_view(),
                    pattern_index,
                    *sub_index,
                )
            )
            .ok_or(NoMatch::NoChildPatterns)
            .and_then(|(pattern_index, sub_index)| {
                self.compare_child_pattern_at_offset(
                    child_patterns,
                    context,
                    pattern_index,
                    sub_index,
                )
                .map(|(parent_match, end_index)|
                    (parent_match, PatternLocation {
                        parent: Child::new(parent_index, parent.width),
                        pattern_id: pattern_index,
                        range: sub_index..end_index
                    })
                )
            })
    }
    /// comparison on child pattern and context
    pub(crate) fn compare_child_pattern_at_offset(
        &'g self,
        child_patterns: &'g ChildPatterns,
        context: impl IntoPattern<Item = impl AsChild>,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> Result<(ParentMatch, usize), NoMatch> {
        let child_pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, rem) = D::directed_pattern_split(child_pattern, sub_index);
        let child_tail = D::pattern_tail(&rem[..]);
        // match search context with child tail
        // back context is from child pattern
        self.compare(context, child_tail).map(|pm| (ParentMatch {
            parent_range: D::to_found_range(pm.1, back_context),
            remainder: pm.0,
        }, child_pattern.len()-pm.1.map(|p| p.len()).unwrap_or_default()))
        // returns result of matching sub with parent's children
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
