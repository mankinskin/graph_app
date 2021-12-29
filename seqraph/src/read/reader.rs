use crate::{
    r#match::*,
    search::*,
    merge::*,
    *,
};
use itertools::*;
use std::num::NonZeroUsize;

#[derive(Debug)]
struct ReaderCache {
    pub(crate) index: Child,
    pub(crate) pattern_id: Option<PatternId>,
}
impl ReaderCache {
    fn new<T: Tokenize + std::fmt::Display>(
        graph: &'_ mut Hypergraph<T>,
        new: impl IntoIterator<Item = Child>,
    ) -> Self {
        let (index, pattern_id) = graph.insert_pattern_with_id(new);
        Self { index, pattern_id }
    }
    fn update_index<T: Tokenize + std::fmt::Display>(
        &mut self,
        graph: &'_ mut Hypergraph<T>,
        new: impl IntoIterator<Item = Child>,
    ) {
        if let Some(pid) = &self.pattern_id {
            self.index = graph.append_to_pattern(self.index, *pid, new);
        } else {
            let (index, pattern_id) =
                graph.insert_pattern_with_id(std::iter::once(self.index).chain(new));
            self.index = index;
            self.pattern_id = pattern_id;
        }
    }
}
#[derive(Debug)]
pub struct Reader<'a, T: Tokenize, D: MatchDirection> {
    graph: &'a mut Hypergraph<T>,
    cache: Option<ReaderCache>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a, T: Tokenize, D: MatchDirection> std::ops::Deref for Reader<'a, T, D> {
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
            cache: None,
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
            .map(|t| match self.get_token_index(&t) {
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
    fn update_cache_index(
        &mut self,
        new: impl IntoIterator<Item = Child>,
    ) {
        if let Some(cache) = &mut self.cache {
            cache.update_index(self.graph, new)
        } else {
            self.cache = Some(ReaderCache::new(self.graph, new));
        }
        println!(
            "Cache index contains: {:?}",
            self.cache
                .as_ref()
                .map(|c| self.graph.index_string(c.index))
                .unwrap_or_default()
        );
    }
    pub(crate) fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> Child {
        let sequence: NewTokenIndices = self.new_token_indices(sequence);
        self.try_read_sequence(sequence).expect("Empty sequence")
    }
    fn try_read_sequence(
        &mut self,
        sequence: NewTokenIndices,
    ) -> Option<Child> {
        if sequence.is_empty() {
            return None;
        }
        let (new, known, rem,) = self.find_known_block(sequence);
        self.update_cache_index(new);
        let known_str = self.graph.pattern_string(&known);
        let rem_str = self.graph.pattern_string(&rem);
        if let Some(cache) = &self.cache {
            println!("cache: \"{}\"", self.graph.index_string(&cache.index));
        }
        println!("known: \"{}\"\nrem: \"{}\"", known_str, rem_str);
        let (first, known) = self.read_prefix(known);
        let (second, known) = self.read_prefix(known);
        let overlaps = self.find_overlaps(first, second);
        self.update_cache_index(first);
        let rem = known.into_iter().map(|c| NewTokenIndex::Known(*c.index()))
            .chain(rem.into_iter()).collect();
        let res = self.try_read_sequence(rem);
        if res.is_none() {
            self.cache.as_ref().map(|c| c.index)
        } else {
            res
        }
    }
    fn read_prefix(
        &mut self,
        pattern: Vec<Child>,
    ) -> (Child, Pattern) {
        let _pat_str = self.graph.pattern_string(&pattern);
        match self.find_ancestor(&pattern) {
            Ok(SearchFound {
                index,
                parent_match,
                ..
            }) => (match parent_match.parent_range {
                FoundRange::Complete => {
                    println!("Found full index");
                    index
                }
                FoundRange::Prefix(post) => {
                    println!("Found prefix");
                    let width = index.width - pattern_width(post);
                    let pos = NonZeroUsize::new(width)
                        .expect("returned full length postfix remainder");
                    let (l, _) = self.index_prefix(index, pos);
                    l
                }
                FoundRange::Postfix(pre) => {
                    println!("Found postfix");
                    let width = pattern_width(pre);
                    let pos = NonZeroUsize::new(width)
                        .expect("returned zero length prefix remainder");
                    let (_, r) = self.index_postfix(index, pos);
                    r
                }
                FoundRange::Infix(pre, post) => {
                    println!("Found infix");
                    let pre_width = pattern_width(pre);
                    let post_width = pattern_width(post);
                    if pre_width == 0 {
                        let pos = NonZeroUsize::new(index.width - post_width)
                            .expect("returned zero length postfix remainder");
                        let (l, _) = self.index_prefix(index, pos);
                        l
                    } else {
                        self.index_subrange(index, pre_width..index.width - post_width)
                    }
                }
            }, parent_match.remainder.unwrap_or_default()),
            Err(_not_found) => {
                let (c, rem) = Left::split_context_head(pattern).unwrap();
                (c, rem)
            },
            //match not_found {
            //    NoMatch::NoMatchingParent(index) => {
            //        // create new index for this known block
            //        let index_str = self.graph.index_string(index);
            //        println!("No matching parents for {}", pat_str);
            //        println!("At index \'{}\'", index_str);
            //        println!("Inserting new pattern");
            //        let c = self.graph.insert_pattern(pattern);
            //        SplitSegment::Child(c)
            //    }
            //    _ => panic!("Not found {:?}", not_found),
            //},
        }
    }
    /// find overlap of non-neighboring children (no common parent)
    fn find_overlaps(
        &mut self,
        left: impl PatternSource,
        right: impl PatternSource,
    ) -> Vec<Pattern> {
        let mut overlaps = Vec::new();
        let rfull = right.full_index();
        let lps = left.back_sorted_patterns(self);
        let rps = right.front_sorted_patterns(self);
        let flp = lps.first().unwrap();
        let fl = Left::get_ordering_element(&flp);
        let fr = Right::get_ordering_element(&rps.first().unwrap());

        let largest = vec![*fl, *fr];
        if let Ok(_) = self.left_searcher().find_parent(&largest) {
            // largest overlap matches
            overlaps.push(largest);
            overlaps
        } else {
            // try smallest
            let slp = lps.last().unwrap();
            let srp = rps.last().unwrap();
            let sl = Left::get_ordering_element(&slp);
            let (sr, srctx) = Right::split_context_head(srp.clone()).unwrap();
            if self.find_parent(&[sl, &sr][..]).is_err() {
                // smallest is not an overlap
                if sl.width() == 1 {
                    if sr.width() == 1 {
                        // no overlap
                        overlaps
                    } else {
                        // move down into right smallest
                        self.find_overlaps(vec![*sl], sr)
                    }
                } else {
                    // move down into left smallest
                    self.find_overlaps(*sl, srp.clone())
                }
            } else {
                // smallest works, largest doesn't
                // search through left largest to smallest
                // for each right smallest to largest
                // Todo: use current result if number of patterns is 1
                for (i, lp) in lps.iter().enumerate() {
                    let (l, lctx) = Left::split_context_head(lp.clone()).unwrap();
                    // find largest overlap with 
                    match rps.iter()
                        // skip largest right if we are largest left
                        .skip(usize::from(i == 0))
                        .rev()
                        .try_fold(None, |rm, rp| {
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
                                let children = miss.vertex(self.graph).get_child_patterns();
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
                        // some left with smallest right doesn't match
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
}
type SortedPatterns = Vec<Pattern>;
trait PatternSource: Sized {
    fn full_index(&self) -> Option<Child>;
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> SortedPatterns;
    fn back_sorted_patterns<T: Tokenize>(self, graph: &Hypergraph<T>) -> SortedPatterns {
        self.sorted_patterns::<T, Left>(graph)
    }
    fn front_sorted_patterns<T: Tokenize>(self, graph: &Hypergraph<T>) -> SortedPatterns {
        self.full_index()
            .into_iter()
            .map(|c| c.into_pattern())
            .chain(
                self.sorted_patterns::<T, Right>(graph)
                    .into_iter()
                )
            .collect()
    }
}
impl PatternSource for Child {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> SortedPatterns {
        D::sort_by_inner(self.vertex(graph).get_child_patterns())
    }
    fn full_index(&self) -> Option<Child> {
        Some(self.clone())
    }
}
impl PatternSource for SortedPatterns {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, _graph: &Hypergraph<T>) -> SortedPatterns {
        self
    }
    fn full_index(&self) -> Option<Child> {
        None
    }
}
impl PatternSource for ChildPatterns {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, _graph: &Hypergraph<T>) -> SortedPatterns {
        D::sort_by_inner(clone_child_patterns(&self))
    }
    fn full_index(&self) -> Option<Child> {
        None
    }
}
impl PatternSource for Pattern {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, _graph: &Hypergraph<T>) -> SortedPatterns {
        vec![self]
    }
    fn full_index(&self) -> Option<Child> {
        None
    }
}
trait OverlapSide {
    /// sorts child patterns by decending width of their most inner child
    fn sort_by_inner(
        mut children: Vec<Pattern>,
    ) -> SortedPatterns {
        children.sort_unstable_by(|a, b|
            Self::get_ordering_element(b).cmp(Self::get_ordering_element(a))
        );
        children
    }
    fn sort_filter_above_width(
        children: Vec<Pattern>,
        min_width: usize,
    ) -> Vec<Pattern> {
        Self::sort_by_inner(children)
            .into_iter()
            .take_while(|p|
                Self::get_ordering_element(p).width() > min_width
            )
            .collect()
    }
    fn get_ordering_element(
        pattern: &Pattern,
    ) -> &Child;
}
impl OverlapSide for Left {
    fn get_ordering_element(
        pattern: &Pattern,
    ) -> &Child {
        pattern.last().unwrap()
    }
}

impl OverlapSide for Right {
    fn get_ordering_element(
        pattern: &Pattern,
    ) -> &Child {
        pattern.first().unwrap()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use maplit::hashset;
    use std::collections::HashSet;
    #[test]
    fn find_overlaps_1() {
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
        assert!(graph.find_parent([b,y]).is_ok());
        assert!(graph.find_parent([b, yz]).is_ok());
        assert!(graph.find_parent([ab, y]).is_ok());
        let overlaps: HashSet<_> = graph.left_reader().find_overlaps(xab, yzw).into_iter().collect();
        assert_eq!(
            overlaps,
            hashset![
                vec![x, aby, z, w],
                vec![xa, byz, w],
            ]
        )
    }
}