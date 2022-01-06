use crate::{
    r#match::*,
    search::*,
    merge::*,
    *,
};
use itertools::*;
use std::num::NonZeroUsize;

#[derive(Debug, Default)]
struct ReaderCache {
    pub(crate) index: Option<(Child, PatternId)>,
    buffer: Option<Child>,
}
impl ReaderCache {
    fn append<T: Tokenize + std::fmt::Display>(
        &mut self,
        graph: &'_ mut Hypergraph<T>,
        new: impl IntoPattern<Item = impl AsChild>,
    ) {
        let new = new.into_pattern();
        if let Some((index, pid)) = self.index.as_mut() {
            *index = graph.append_to_pattern(index.clone(), *pid, new);
        } else {
            match new.len() {
                0 => {},
                1 => {
                    let new = new.into_iter().next().unwrap();
                    if let Some(buffer) = self.buffer.take() {
                        let (index, pid) = graph.insert_pattern_with_id([buffer, new]);
                        self.index = Some((index, pid.unwrap()));
                    } else {
                        self.buffer = Some(new);
                    }
                },
                _ => {
                    let new = if let Some(buffer) = self.buffer.take() {
                        [&[buffer], &new[..]].concat()
                    } else {
                        new
                    };
                    let (index, pid) = graph.insert_pattern_with_id(new);
                    self.index = Some((index, pid.unwrap()));
                }
            }
        }
    }
    fn get(&self) -> Option<Child> {
        self.index.map(|(i, _)| i).or(self.buffer)
    }
}
#[derive(Debug)]
pub struct Reader<'a, T: Tokenize, D: MergeDirection> {
    graph: &'a mut Hypergraph<T>,
    cache: ReaderCache,
    _ty: std::marker::PhantomData<D>,
}
impl<'a, T: Tokenize, D: MergeDirection> std::ops::Deref for Reader<'a, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize, D: MatchDirection> std::ops::DerefMut for Reader<'a, T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize + std::fmt::Display, D: MatchDirection> Reader<'a, T, D> {
    pub(crate) fn new(graph: &'a mut Hypergraph<T>) -> Self {
        Self {
            graph,
            cache: ReaderCache::default(),
            _ty: Default::default(),
        }
    }
    #[allow(unused)]
    pub(crate) fn right_searcher(&self) -> Searcher<T, Right> {
        Searcher::new(self.graph)
    }
    fn new_token_indices(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> NewTokenIndices {
        sequence
            .into_iter()
            .map(|t| Token::Element(t))
            .map(|t| match self.get_token_index(t) {
                Ok(i) => NewTokenIndex::Known(i),
                Err(_) => {
                    let i = self.insert_token(t);
                    NewTokenIndex::New(i.index)
                }
            })
            .collect()
    }
    fn take_while<I, J: Iterator<Item = I> + itertools::PeekingNext>(
        iter: &mut J,
        f: impl FnMut(&I) -> bool,
    ) -> Pattern
    where
        Child: From<I>,
    {
        iter.peeking_take_while(f).map(Child::from).collect()
    }
    fn find_known_block(
        &mut self,
        sequence: NewTokenIndices,
    ) -> (Pattern, Pattern, NewTokenIndices) {
        let mut seq_iter = sequence.into_iter().peekable();
        let cache = Self::take_while(&mut seq_iter, |t| t.is_new());
        let known = Self::take_while(&mut seq_iter, |t| t.is_known());
        (cache, known, seq_iter.collect())
    }
    pub(crate) fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> Child {
        let sequence: NewTokenIndices = self.new_token_indices(sequence);
        self.try_read_sequence(sequence)
    }
    fn try_read_sequence(
        &mut self,
        sequence: NewTokenIndices,
    ) -> Child {
        if sequence.is_empty() {
            return self.cache.get().expect("Empty sequence");
        }
        let (new, mut known, remainder) = self.find_known_block(sequence);
        //let known_str = self.graph.pattern_string(&known);
        //let rem_str = self.graph.pattern_string(&rem);
        //if let Some(cache) = &self.cache {
        //    println!("cache: \"{}\"", self.graph.index_string(&cache.index));
        //}
        //println!("known: \"{}\"\nrem: \"{}\"", known_str, rem_str);
        self.cache.append(self.graph, new);
        // read indicies in known
        let mut patterns: Vec<Pattern> = Vec::new();
        // - single index added to patterns
        // |   *   |
        // 2. next index added
        // |       |    *    |
        // - find overlaps with previous patterns
        // - index context on left side of overlap
        // - get 2 or 3 children back per overlap
        // |       |         |
        // | * |        |    | no full match with right index
        // |  *  |           | full match with right index
        // - (recognize shared borders in pat #1 and #3 -> create new index)
        // - find next index (go to step 2)
        // |       |         |     *    |
        // |   |        |    |
        // |     |           |
        // - find overlaps with previous patterns and new index
        // - index context on left side of overlap
        // |       |         |          |
        // |    *       |        |      | right index of prev overlap continues to match
        // |     |           |            not here
        // - recognize shared borders in pat #1 and #3 -> create new index
        // |       *         |          |
        // |   |        |        |      | right index of prev overlap continues to match
        // - continue at step 2.

        let mut group = None;
        let mut buffer = None;
        while !known.is_empty() {
            let (next, rem) = self.read_prefix(known);
            if let Some(group) = group {

            } else {
                // first or second index
                if let Some(buffer) = buffer.take() {
                    // second index
                    group = Some(PatternGroup::new(buffer, next));
                } else {
                    // first index
                    buffer = Some(next);
                }
            }
            known = rem;
            //match patterns.len() {
            //    // add first index
            //    0 => patterns.push(vec![next]),
            //    // find overlaps with single pattern
            //    1 => {
            //        // prevent multiple indicies in single pattern
            //        let p = patterns.pop().unwrap();
            //        // split last
            //        let (rem, last) = D::split_last(p).unwrap();
            //        self.cache.append(self.graph, rem);
            //        let overlaps = self.find_overlaps(last, next);
            //        if overlaps.is_empty() {
            //            self.cache.append(self.graph, last);
            //            patterns.push(vec![next]);
            //        } else {
            //            patterns.push(vec![last, next]);
            //            patterns.extend(overlaps);
            //        }
            //    },
            //    _ => {
            //        let overlaps = self.find_overlaps(Left::sort_by_inner(patterns.clone()), next);
            //        if overlaps.is_empty() {
            //            let new = self.insert_patterns(patterns);
            //            patterns = vec![vec![new, next]];
            //        } else {
            //            // Todo: separate overlap patterns and non overlap patterns
            //            patterns[0].push(next);
            //            patterns = vec![patterns.swap_remove(0)];
            //            patterns.extend(overlaps);
            //        }
            //    }
            //}
        }
        //match patterns.len() {
        //    0 => {},
        //    1 => {
        //        let pattern = patterns.into_iter().next().unwrap();
        //        self.cache.append(self.graph, pattern);
        //    }
        //    _ => {
        //        let pattern = self.insert_patterns(patterns).into_pattern();
        //        self.cache.append(self.graph, pattern);
        //    }
        //}
        self.try_read_sequence(remainder)
    }
    fn read_search_found(
        &mut self,
        search_found: SearchFound,
    ) -> (Child, Pattern) {
        let SearchFound {
                location: PatternLocation {
                    parent: index,
                    ..
                },
                parent_match,
                ..
            } = search_found;
        (match parent_match.parent_range {
                FoundRange::Complete => {
                    //println!("Found full index {}", self.graph.index_string(&index));
                    index
                }
                FoundRange::Prefix(post) => {
                    //println!("Found prefix of {} width {}", self.graph.index_string(&index), index.width);
                    let width = pattern_width(&post);
                    //println!("postfix {} width {}", self.graph.pattern_string(&post), width);
                    //println!("{:#?}", &post);
                    let width = index.width - width;
                    let pos = NonZeroUsize::new(width)
                        .expect("returned full length postfix remainder");
                    //println!("prefix width {}", pos.get());
                    let (l, _) = self.index_prefix(index, pos);
                    l
                }
                FoundRange::Postfix(pre) => {
                    //println!("Found postfix of {}", self.graph.index_string(&index));
                    //println!("prefix {}", self.graph.pattern_string(&pre));
                    let width = pattern_width(pre);
                    let pos = NonZeroUsize::new(width)
                        .expect("returned zero length prefix remainder");
                    let (_, r) = self.index_postfix(index, pos);
                    r
                }
                FoundRange::Infix(pre, post) => {
                    //println!("Found infix of {}", self.graph.index_string(&index));
                    //println!("postfix {}", self.graph.pattern_string(&post));
                    //println!("prefix {}", self.graph.pattern_string(&pre));
                    let pre_width = pattern_width(pre);
                    let post_width = pattern_width(post);
                    //println!("{}, {}, {}", pre_width, post_width, index.width);
                    //println!("{}", self.index_string(index));
                    self.index_subrange(index, pre_width..index.width - post_width)
                }
            }, parent_match.remainder.unwrap_or_default())
    }
    fn read_prefix(
        &mut self,
        pattern: Vec<Child>,
    ) -> (Child, Pattern) {
        let _pat_str = self.graph.pattern_string(&pattern);
        match self.find_ancestor(&pattern) {
            Ok(search_found) => self.read_search_found(search_found),
            Err(_not_found) => {
                let (c, rem) = Right::split_context_head(pattern).unwrap();
                (c, rem)
            },
        }
    }
    // child child: go into left children, match with right and right children
    //      use left to update parent
    // patterns child: run without left parent, find overlaps for each pattern with id
    // child patterns: go into left children, match with right patterns
    // patterns patterns: run without left parent
    /// get largest overlap 
    fn overlap_largest(
        &mut self,
        left_parent: Option<Child>,
        left_id: PatternId,
        left_ctx: Pattern,
        left: Child,
        right: Child,
        right_ctx: Pattern,
    ) -> Option<PatternLocation> {
        let largest = vec![left, right];
        self.right_searcher().find_parent(&largest)
            .map(|search_found| {
                // largest overlap matches
                // index overlap
                // index contexts in left and right
                let (index, _rem) = self.read_search_found(search_found);
                let pre = if left_ctx.len() > 1 {
                    if let Some(lpar) = left_parent {
                        let lctx = self.index_range_in(lpar, left_id, 0..left_ctx.len());
                        lctx.into_pattern()
                    } else {
                        left_ctx
                    }
                } else {
                    left_ctx
                };
                (left_id, [&pre[..], &[index][..], &right_ctx[..]].concat())
            })
            .ok()
    }
    // - left complete must not match with any right prefix
    // - match largest left postfix with complete right
    //     - if match, index left context and return resulting pattern
    // - otherwise call find overlaps on right children
    fn find_overlaps(
        &mut self,
        left: impl PatternSource,
        right: impl PatternSource,
    ) -> Overlaps {
        let mut overlaps = Vec::new();
        let lps = left.sorted_patterns(self);
        // left pattern with largest child
        let (flp_id, flp) =
            if let Some(l) = lps.first() {
                l
            } else {
                return vec![];
            };
        let lparent = left.get_parent();
        // split off largest match child
        let (fl, fl_rem) = Left::split_ordering_element(flp);
        let (rlargest, rctx) = if let Some(rlargest) = right.get_parent() {
            (rlargest, &[][..])
        } else {
            let rps = right.sorted_patterns(self);
            let (frp_id, frp) =
                if let Some(r) = rps.first() {
                    r
                } else {
                    return vec![];
                };
            Right::split_ordering_element(frp)
        };

        // todo handle right child pattern ids
        if let Some(pattern) = self.overlap_largest(
            lparent, *flp_id, fl_rem.into_pattern(), fl, rlargest, rctx.into_pattern(),
        ) {
            vec![pattern]
        } else {
            // largest doesn't match
            // try smallest
            let (slp_id, slp) = lps.last().unwrap();
            let (srp_id, srp) = rps.last().unwrap();
            let (sl, slctx) = Left::split_ordering_element(slp);
            let (sr, srctx) = Right::split_ordering_element(srp);
            if self.find_parent(&[sl, sr][..]).is_err() {
                // smallest is not an overlap
                if sl.width() == 1 {
                    if sr.width() == 1 {
                        // no overlap
                        overlaps
                    } else {
                        // move down into right smallest
                        self.find_overlaps((left.get_parent(), *slp_id, *slp), sr)
                    }
                } else {
                    // move down into left smallest
                    self.find_overlaps(sl, right)
                }
            } else {
                // smallest works, largest doesn't
                // search through left largest to smallest
                // for each right smallest to largest
                // Todo: use current result if number of patterns is 1
                for (i, (lpid, lp)) in lps.iter().enumerate() {
                    let (l, lctx) = Left::split_context_head(lp.clone()).unwrap();
                    // find largest overlap with 
                    match rps.iter()
                        .map(|(pid, p)| (Some(pid), p))
                        .rev()
                        // add largest right if we are not largest left
                        .chain((i != 0).then(|| (None, &right.into_pattern())))
                        .try_fold(None, |rm, (rpid, rp)| {
                            let (r, rctx) = Right::split_context_head(rp.clone()).unwrap();
                            match self.find_parent([l, r].as_slice()) {
                                Ok(f) => Ok(Some(((r, rctx), f))),
                                Err(_) => Err(rm.map(|found| ((r, rctx), found))),
                            }
                        }) {
                        Ok(Some(((r, rctx), found))) => {
                            // largest right matches
                            // extract pattern
                            // add overlap
                            let overlap = self.index_overlap(&[l, r][..], found, lctx, rctx);
                            overlaps.push(overlap);
                        }
                        Err(Some((
                            (miss, _),
                            (
                                (r, rctx),
                                found
                            )))) => {
                            // some found right matches
                            // larger miss didn't match
                            // find rights in miss' children larger than found
                            if Some(miss) == rfull {
                                // next biggest is the full index, i.e. largest right matches
                                let overlap = self.index_overlap(&[l, r][..], found, lctx, rctx);
                                overlaps.push(overlap);
                            } else {
                                let children = miss.vertex(self.graph).get_child_pattern_vec();
                                let candidates =
                                    Right::sort_filter_above_width(children, r.width());
                                let olps = self.find_overlaps(l, candidates);
                                if olps.is_empty() {
                                    let overlap = self.index_overlap(&[l, r][..], found, lctx, rctx);
                                    overlaps.push(overlap);
                                } else {
                                    overlaps.extend(olps);
                                }
                            }
                        }
                        // this left with smallest right doesn't match
                        Ok(None) | Err(None) => {
                            let olps = self.find_overlaps(vec![l], sr);
                            overlaps.extend(olps.into_iter().map(|olp| {
                                let olp = self.merge_left_split(lctx.clone(), olp.into());
                                self.merge_right_split(srctx.clone(), olp.into()).into_pattern()
                            }));
                        },
                    }
                }
                overlaps
            }
        }
    }
    fn index_overlap(
        &mut self,
        pattern: impl IntoPattern<Item=impl AsChild>,
        found: SearchFound,
        lctx: Pattern,
        rctx: Pattern,
    ) -> Pattern {
        let new = self.index_found(pattern, found);
        Right::concat_inner_and_context(lctx, vec![new], rctx)
    }
    fn new_pattern_group(
        &self,
        first: Child,
        second: Child,
    ) -> PatternGroup {
        let overlaps = self.find_overlaps(first, second);
        PatternGroup::overlapped(first, second, overlaps)
    }
}
struct PatternGroup {
    prefix: Pattern,
    overlapped: Vec<PatternGroup>,
    overlaps: Overlaps,
}
type Overlaps = Vec<Overlap>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PatternLocation {
    pub parent: Child,
    pub pattern_id: PatternId,
    pub range: std::ops::Range<usize>,
}
enum Overlap {
    Expandable(PatternLocation, PatternLocation, PatternLocation),
    EndPiece(PatternLocation, PatternLocation),
}
impl PatternGroup {
    pub fn single(first: Child) -> Self {
        Self {
            prefix: vec![first],
            overlapped: vec![],
            overlaps: vec![],
        }
    }
    pub fn overlapped(first: Child, second: Child, overlaps: Overlaps) -> Self {
        Self {
            prefix: vec![],
            overlapped: vec![Self::single(first), Self::single(second)],
            overlaps,
        }
    }
}
// - has parent if patterns are stored in other index
// - patterns have id if they are stored in other index
type SortedPatterns = Vec<(PatternId, Pattern)>;
trait PatternSource: Sized {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> SortedPatterns;
    fn get_parent(&self) -> Option<Child>;
}
//trait LeftPatternSource: Sized + PatternSource {
//    fn get_parent(&self) -> Option<Child>;
//}
impl PatternSource for Child {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> SortedPatterns {
        if self.width > 1 {
            D::sort_by_inner(self.vertex(graph).get_children())
        } else {
            vec![]
        }
    }
    //fn full_index(&self) -> Option<Child> {
    //    Some(self.clone())
    //}
    fn get_parent(&self) -> Option<Child> {
        Some(*self)
    }
}
impl PatternSource for SortedPatterns {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, _graph: &Hypergraph<T>) -> SortedPatterns {
        self
    }
    fn get_parent(&self) -> Option<Child> {
        None
    }
    //fn full_index(&self) -> Option<Child> {
    //    self.first().and_then(|p| p.full_index())
    //}
}
//impl PatternSource for Pattern {
//    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, _graph: &Hypergraph<T>) -> SortedPatterns {
//        vec![self]
//    }
//    fn full_front_index(&self) -> Option<Child> {
//        if self.len() == 1 {
//            self.first().cloned()
//        } else {
//            None
//        }
//    }
//    fn full_back_index(&self) -> Option<Child> {
//        if self.len() == 1 {
//            self.first().cloned()
//        } else {
//            None
//        }
//    }
//}
//impl PatternSource for ChildPatterns {
//    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> SortedPatterns {
//        D::sort_by_inner(clone_child_patterns(&self).into_iter().collect()).sorted_patterns::<T, D>(graph)
//    }
//    fn full_front_index(&self) -> Option<Child> {
//        if self.len() == 1 {
//            self.into_iter().next().and_then(|p| p.1.full_front_index())
//        } else {
//            None
//        }
//    }
//    fn full_back_index(&self) -> Option<Child> {
//        if self.len() == 1 {
//            self.into_iter().next().and_then(|p| p.1.full_back_index())
//        } else {
//            None
//        }
//    }
//}
trait OverlapSide : MatchDirection {
    type Opposite: OverlapSide;
    /// sorts child patterns by decending width of their most inner child
    fn sort_by_inner(
        children: &ChildPatterns,
    ) -> SortedPatterns {
        let mut children = children.clone().into_iter().collect_vec();
        children.sort_unstable_by(|a, b|
            Self::get_ordering_element(&b.1).cmp(Self::get_ordering_element(&a.1))
        );
        children
    }
    fn sort_filter_above_width(
        children: &ChildPatterns,
        min_width: usize,
    ) -> SortedPatterns {
        Self::sort_by_inner(children)
            .into_iter()
            .take_while(|p|
                Self::get_ordering_element(&p.1).width() > min_width
            )
            .collect()
    }
    fn get_ordering_element(
        pattern: &Pattern,
    ) -> &Child {
        <Self as MatchDirection>::pattern_head(pattern).expect("Empty pattern in overlap check!")
    }
    fn split_ordering_element(
        pattern: &Pattern,
    ) -> (Child, PatternView<'_>) {
        <Self as MatchDirection>::split_head_tail(pattern).expect("Empty pattern in overlap check!")
    }
}
impl OverlapSide for Left {
    type Opposite = Right;
}

impl OverlapSide for Right {
    type Opposite = Left;
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use maplit::hashset;
    use std::collections::HashSet;
    #[test]
    fn find_overlaps1() {
        let mut graph = Hypergraph::default();
        let (a, b, x, y, z, w) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
            Token::Element('w'),
        ]).into_iter().next_tuple().unwrap();
        let ab = graph.insert_pattern([a, b]);
        let by = graph.insert_pattern([b, y]);
        let aby = graph.insert_patterns([[ab, y], [a, by]]);
        let yz = graph.insert_pattern([y, z]);
        let xa = graph.insert_pattern([x, a]);
        let xab = graph.insert_patterns([[x, ab], [xa, b]]);
        let byz = graph.insert_patterns([[by, z], [b, yz]]);
        let yzw = graph.insert_pattern([yz, w]);
        //let xaby = graph.insert_patterns([[xab, y], [xa, by]]);
        //let _xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
        assert!(graph.find_parent([b, y]).is_ok());
        assert!(graph.find_parent([b, yz]).is_ok());
        assert!(graph.find_parent([ab, y]).is_ok());
        let overlaps: HashSet<_> = graph.right_reader().find_overlaps(xab, yzw).into_iter().collect();
        assert_eq!(
            overlaps,
            hashset![
                vec![x, aby, z, w],
                vec![xa, byz, w],
            ]
        )
    }
    #[test]
    fn find_overlaps2() {
        let mut graph = Hypergraph::default();
        let (v, i, s, u, b)= graph.insert_tokens([
            Token::Element('v'),
            Token::Element('i'),
            Token::Element('s'),
            Token::Element('u'),
            Token::Element('b'),
        ]).into_iter().next_tuple().unwrap();
        let vis = graph.insert_pattern([v, i, s]);
        let sub = graph.insert_pattern([s, u, b]);
        assert!(graph.find_parent([s, u]).is_ok());
        assert!(graph.find_parent([v, i]).is_ok());
        let overlaps: HashSet<_> = graph.right_reader().find_overlaps(vis, u).into_iter().collect();
        let vi = graph.find_sequence("vi".chars()).unwrap().expect_complete("vi");
        let pats = vi.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![v, i],
        ]);
        let su = graph.find_sequence("su".chars()).unwrap().expect_complete("su");
        let pats = su.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![s, u],
        ]);
        let pats = vis.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![vi, s],
        ]);
        let pats = sub.vertex(&graph).get_child_pattern_set();
        assert_eq!(pats, hashset![
            vec![su, b],
        ]);
        assert_eq!(
            overlaps,
            hashset![
                vec![vi, su],
            ]
        )
    }
}