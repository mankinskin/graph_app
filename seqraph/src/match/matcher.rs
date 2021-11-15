use crate::{
    r#match::*,
    Child,
    ChildPatterns,
    Hypergraph,
    Indexed,
    Parent,
    PatternId,
    TokenPosition,
};
use itertools::*;
use std::borrow::Borrow;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PatternMatch(pub Option<Pattern>, pub Option<Pattern>);

impl PatternMatch {
    pub fn left(&self) -> &Option<Pattern> {
        &self.0
    }
    pub fn right(&self) -> &Option<Pattern> {
        &self.1
    }
    pub fn flip_remainder(self) -> Self {
        Self(self.1, self.0)
    }
    pub fn is_matching(&self) -> bool {
        self.left().is_none() && self.right().is_none()
    }
}
impl From<Either<Pattern, Pattern>> for PatternMatch {
    fn from(e: Either<Pattern, Pattern>) -> Self {
        match e {
            Either::Left(p) => Self(Some(p), None),
            Either::Right(p) => Self(None, Some(p)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParentMatch {
    pub parent_range: FoundRange,
    pub remainder: Option<Pattern>,
}
impl ParentMatch {
    pub fn embed_in_super(self, other: Self) -> Self {
        Self {
            parent_range: self.parent_range.embed_in_super(other.parent_range),
            remainder: other.remainder,
        }
    }
}
pub type PatternMatchResult = Result<PatternMatch, NoMatch>;
pub type ParentMatchResult = Result<ParentMatch, NoMatch>;

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
        A: IntoPattern<Item = impl Into<Child> + Tokenize>,
        B: IntoPattern<Item = impl Into<Child> + Tokenize>,
    >(
        &self,
        a: A,
        b: B,
    ) -> PatternMatchResult {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        let a: Pattern = a.into_pattern();
        let b: Pattern = b.into_pattern();
        if let Some((pos, eob)) = D::skip_equal_indices(a.clone().iter(), b.clone().iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(ai, bi) => {
                    self.match_unequal_indices_with_remainders(*ai, a, *bi, b, pos)
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
    fn decide_sub_and_sup_indicies<
        A: IntoPattern<Item = impl Into<Child> + Tokenize, Token = impl Into<Child> + Tokenize>,
    >(
        a: Child,
        a_context: A,
        b: Child,
        b_context: A,
    ) -> Result<(bool, Child, A, Child, A), NoMatch> {
        // remember if sub and sup were switched
        match a.width.cmp(&b.width) {
            // relatives can not have same sizes
            Ordering::Equal => Err(NoMatch::Mismatch),
            Ordering::Less => {
                //println!("right super");
                Ok((false, a, a_context, b, b_context))
            }
            Ordering::Greater => {
                //println!("left super");
                Ok((true, b, b_context, a, a_context))
            }
        }
    }
    #[allow(unused)]
    fn match_indices(&self, a: Child, b: Child) -> PatternMatchResult {
        if a == b {
            return Ok(PatternMatch(None, None));
        }
        self.match_unequal_indices_in_context(a, &[] as &[Child], b, &[])
    }
    fn match_unequal_indices_with_remainders<
        C: Into<Child> + Tokenize,
        A: IntoPattern<Item = impl Into<Child> + Tokenize, Token = C>,
        B: IntoPattern<Item = impl Into<Child> + Tokenize, Token = C>,
    >(
        &self,
        a: Child,
        a_pattern: A,
        b: Child,
        b_pattern: B,
        pos: TokenPosition,
    ) -> PatternMatchResult {
        let a_context = D::front_context_normalized(a_pattern.as_pattern_view(), pos);
        let b_context = D::front_context_normalized(b_pattern.as_pattern_view(), pos);
        self.match_unequal_indices_in_context(a, a_context, b, b_context)
    }
    fn match_unequal_indices_in_context<
        C: Into<Child> + Tokenize,
        A: IntoPattern<Item = impl Into<Child> + Tokenize, Token = C>,
    >(
        &self,
        a: Child,
        a_context: A,
        b: Child,
        b_context: A,
    ) -> PatternMatchResult {
        // Note: depending on sizes of a, b it may be differently efficient
        // to search for children or parents, large patterns have less parents,
        // small patterns have less children
        // search larger in parents of smaller
        let (rotate, sub, sub_context, sup, sup_context) =
            Self::decide_sub_and_sup_indicies(a, a_context, b, b_context)?;
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
    fn match_exactly(
        sub: impl Indexed,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        sup: Child,
    ) -> Option<ParentMatch> {
        (*sub.index() == *sup.index()).then(|| ParentMatch {
            parent_range: FoundRange::Complete,
            remainder: (!context.is_empty()).then(|| context.into_pattern()),
        })
    }
    /// match sub index and context with sup index with max width
    #[allow(unused)]
    fn match_sub_and_context_with_index(
        &self,
        sub: impl Indexed,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        sup: Child,
    ) -> ParentMatchResult {
        Self::match_exactly(
            sub.index(),
            context.as_pattern_view(),
            sup,
        )
        .map(Ok)
        .unwrap_or_else(||
            self.match_sub_vertex_and_context_with_index(
                self.expect_vertex_data(sub),
                context,
                sup,
            )
        )
    }
    fn find_unequal_matching_ancestor(
        &self,
        sub: impl Indexed,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        sup: Child,
    ) -> ParentMatchResult {
        let sub_index = *sub.index();
        let vertex = self.expect_vertex_data(sub);
        self.match_sub_vertex_and_context_with_index(
            vertex,
            context.as_pattern_view(),
            sup,
        )
        .or_else(|_|
            self
            .searcher()
            .find_largest_matching_parent_below_width(
                vertex,
                context,
                Some(sup.width),
            )
            .and_then(
                |SearchFound {
                     index: parent_index,
                     parent_match: ParentMatch {
                         parent_range,
                         remainder,
                     },
                     ..
                }|
                D::found_at_start(parent_range)
                    .then(|| remainder.unwrap_or_default())
                    .ok_or(NoMatch::NoMatchingParent(sub_index))
                    .and_then(|new_context|
                        self.find_matching_ancestor(
                            parent_index,
                            new_context,
                            sup,
                        )
                    )
            )
        )
    }
    #[allow(unused)]
    fn find_matching_ancestor(
        &self,
        sub: impl Indexed,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        sup: Child,
    ) -> ParentMatchResult {
        // sup is no direct parent, search upwards
        //println!("matching available parents");
        // search sup in parents
        Self::match_exactly(sub.index(), context.as_pattern_view(), sup)
            .map(Ok)
            .unwrap_or_else(|| self.find_unequal_matching_ancestor(sub, context, sup))
    }
    fn match_sub_vertex_and_context_with_index(
        &self,
        vertex: &VertexData,
        sub_context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        sup: Child,
    ) -> ParentMatchResult {
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
                    sub_context,
                    sup,
                    parent,
                )
                .map(|(parent_match, _, _)| parent_match)
            )
    }
    /// match context against child context in parent.
    pub fn match_context_with_parent_children(
        &'g self,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Result<(ParentMatch, PatternId, usize), NoMatch> {
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent_index);
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub index
        let candidates = D::filter_parent_pattern_indices(parent, child_patterns);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        Self::get_best_child_pattern(
            child_patterns,
            candidates.iter(),
            context.as_pattern_view()
        )
        .ok_or(NoMatch::NoChildPatterns)
        .and_then(|(pattern_index, sub_index)|
            self.compare_child_pattern_at_offset(
                child_patterns,
                context,
                pattern_index,
                sub_index,
            )
            .map(|parent_match| (parent_match, pattern_index, sub_index))
        )
    }
    /// try to find child pattern with context matching sub_context
    pub(crate) fn get_best_child_pattern(
        child_patterns: &'_ ChildPatterns,
        candidates: impl Iterator<Item = impl Borrow<(usize, PatternId)>>,
        sub_context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
    ) -> Option<(PatternId, usize)> {
        candidates
            .map(|c| *c.borrow())
            .find_or_first(|(pattern_index, sub_index)| {
                Self::compare_next_index_in_child_pattern(
                    child_patterns,
                    sub_context.as_pattern_view(),
                    pattern_index,
                    *sub_index,
                )
            })
    }
    pub(crate) fn compare_next_index_in_child_pattern(
        child_patterns: &'_ ChildPatterns,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        pattern_index: &PatternId,
        sub_index: usize,
    ) -> bool {
        D::pattern_head(context.as_pattern_view())
            .and_then(|next_sub| {
                let next_sub: Child = (*next_sub).into();
                D::index_next(sub_index).and_then(|i| {
                    child_patterns
                        .get(pattern_index)
                        .and_then(|pattern| pattern.get(i).map(|next_sup| next_sub == *next_sup))
                })
            })
            .unwrap_or(false)
    }
    /// comparison on child pattern and context
    pub fn compare_child_pattern_at_offset(
        &'g self,
        child_patterns: &'g ChildPatterns,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> ParentMatchResult {
        let child_pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, rem) = D::directed_pattern_split(child_pattern, sub_index);
        let child_tail = D::pattern_tail(&rem[..]);
        // match search context with child tail
        // back context is from child pattern
        self.compare(context, child_tail).map(|pm| ParentMatch {
            parent_range: D::to_found_range(pm.1, back_context),
            remainder: pm.0,
        })
        // returns result of matching sub with parent's children
    }
    pub(crate) fn match_indirect_parent(
        &'g self,
        index: VertexIndex,
        parent: &Parent,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
    ) -> Option<SearchFound> {
        self.match_context_with_parent_children(context.as_pattern_view(), index, parent)
            .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                index: Child::new(index, parent.width),
                pattern_id,
                sub_index,
                parent_match,
            })
            .ok()
    }
}
