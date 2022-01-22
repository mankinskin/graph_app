use crate::{
    r#match::*,
    search::*,
    index::*,
    *,
};
use itertools::*;

#[derive(Debug, Default)]
struct ReaderCache {
    pub(crate) group: Option<Child>,
    buffer: Option<Child>,
}
impl ReaderCache {
    fn append_new<'g, T: Tokenize, D: IndexDirection>(
        &mut self,
        reader: &'_ mut Reader<'g, T, D>,
        new: Pattern,
    ) {
        if let Some(group) = self.group.as_mut() {
            *group = reader.append_new_pattern_to_index(group.clone(), new);
        } else {
            match new.len() {
                0 => {},
                1 => {
                    let new = new.into_iter().next().unwrap();
                    if let Some(buffer) = self.buffer.take() {
                        self.group = Some(reader.insert_pattern(vec![buffer, new]));
                    } else {
                        self.group = Some(reader.force_insert_pattern(vec![new]));
                    }
                },
                _ => {
                    let new = if let Some(buffer) = self.buffer.take() {
                        [&[buffer], &new[..]].concat()
                    } else {
                        new
                    };
                    // insert new pattern so it can be found in later queries
                    // any new indicies afterwards need to be appended
                    // to the pattern inside this index
                    let new = reader.insert_pattern(new);
                    self.group = Some(new);
                }
            }
        }
    }
    fn append_next<'g, T: Tokenize, D: IndexDirection>(
        &mut self,
        reader: &'_ mut Reader<'g, T, D>,
        next: Child,
    ) {
        if let Some(group) = self.group.as_mut() {
            *group = reader.append_known_index_to_index(group.clone(), next);
        } else {
            // first or second index
            if let Some(buffer) = self.buffer.take() {
                // second index
                self.group = Some(reader.overlap_sequence(buffer, next));
            } else {
                // first index
                self.buffer = Some(next);
            }
        }
    }
    fn get(self) -> Option<Child> {
        self.group.or(self.buffer)
    }
}
#[derive(Debug)]
pub struct Reader<'a, T: Tokenize, D: IndexDirection> {
    graph: &'a mut Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a, T: Tokenize, D: IndexDirection> std::ops::Deref for Reader<'a, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize, D: IndexDirection> std::ops::DerefMut for Reader<'a, T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize, D: IndexDirection> Reader<'a, T, D> {
    pub(crate) fn new(graph: &'a mut Hypergraph<T>) -> Self {
        Self {
            graph,
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
        self.try_read_sequence(ReaderCache::default(), sequence)
    }
    // read indicies in known
    // 1. find next index
    // |   *   |
    // - if previous PatterGroup ... (no)
    // - if index in buffer ... (no)
    // - then move into buffer
    // 1. find next index
    // ||      ||    *    |
    // - if previous PatterGroup .. (no)
    // - if index in buffer ... (yes)
    // - find overlaps with index in buffer
    // |       |         |
    // |   |        |    |
    // | |   |           |
    // - skip expansion overlaps in previous index
    // - index left context in non expansion overlaps (because this will become a new index)
    // |       |         |
    // |   |        |    |
    // |  *  |           |
    // - build new PatternGroup
    // ||       |         ||
    // ||   |        |    ||
    // ||     |           ||
    // - find next index (go to step 1)
    // ||       |         ||     *    |
    // ||   |        |    ||
    // ||     |           ||
    // - if previous PatterGroup .. (yes)
    // - append next index to previous pattern group (see PatternGroup::append)

    fn try_read_sequence(
        &mut self,
        mut cache: ReaderCache,
        sequence: NewTokenIndices,
    ) -> Child {
        if sequence.is_empty() {
            cache.get().expect("Empty sequence")
        } else {
            let (new, mut known, remainder) = self.find_known_block(sequence);
            cache.append_new(self, new);
            while !known.is_empty() {
                let (next, rem) = self.read_prefix(known);
                cache.append_next(self, next);
                known = rem;
            }
            self.try_read_sequence(cache, remainder)
        }
    }
    fn read_prefix(
        &mut self,
        pattern: Vec<Child>,
    ) -> (Child, Pattern) {
        //let _pat_str = self.graph.pattern_string(&pattern);
        match self.find_ancestor(&pattern) {
            Ok(found_path) => self.index_found(found_path),
            Err(_not_found) => {
                let (c, rem) = D::split_context_head(pattern).unwrap();
                (c, rem)
            },
        }
    }
    pub fn overlap_sequence(&mut self, first: Child, second: Child) -> Child {
        //let overlaps = self.read_overlaps(first, second);
        //self.graph.insert_patterns([&[vec![first, second]], overlaps.as_slice()].concat())
        self.graph.insert_pattern([first, second].as_slice())
    }
    pub fn read_overlaps(&mut self, first: Child, second: Child) -> Vec<Pattern> {
        self.find_overlaps(first, second)
            .into_iter()
            .map(|overlap| overlap.index(self))
            .collect()
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    pub fn append_new_pattern_to_index(
        &mut self,
        parent: Child,
        new: Pattern,
    ) -> Child {
        let vertex = parent.vertex_mut(self);
        if vertex.children.len() == 1 {
            // if no old overlaps
            // append to single pattern
            // no overlaps because new
            let (&pid, _) = vertex.expect_any_pattern();
            self.append_to_pattern(parent, pid, new)
        } else {
            // some old overlaps though
            self.insert_pattern([&[parent], new.as_slice()].concat())
        }
    }
    /// append a pattern of known indices, with overlaps
    /// returns index of possible new index
    pub fn append_known_index_to_index(
        &mut self,
        parent: Child,
        next: Child,
    ) -> Child {
        let vertex = parent.vertex(self);
        if vertex.children.len() == 1 {
            // if no old overlaps
            let (&pid, pat) = vertex.expect_any_pattern();
            let pat = pat.clone();
            //let last = pat.last().unwrap().clone();
            //let overlaps = self.read_overlaps(last.clone(), next);
            //if overlaps.is_empty() {
                // no new overlaps
                // simply append next
            self.append_to_pattern(parent, pid, next)
            //} else {
            //    let new = self.insert_with_overlaps(last.clone(), next, overlaps);
            //    self.replace_in_pattern(parent, pid, pat.len()-1..pat.len(), new);
            //    parent
            //}
        } else {
            // some old overlaps though
            let (_pid, last) = vertex.largest_postfix();
            let _overlaps = self.read_overlaps(last, next);
            // TODO
            //parent
            unimplemented!()
        }
    }
    // - left complete must not match with any right prefix
    // - match largest left postfix with complete right
    //     - if match, index left context and return resulting pattern
    // - otherwise call find overlaps on right children
    fn find_overlap_with_right(
        &mut self,
        lps: &LocatedPatterns,
        right: Child,
    ) -> Option<(Child, PatternLocation, Pattern)> {
        match lps.iter()
            .try_fold(None, |_, (lploc, lp)| {
                let (l, lctx) = Left::split_context_head(lp.clone()).unwrap();
                match self.find_parent([l, right].as_slice()) {
                    Ok(f) => Err(// break
                            (lploc, l, lctx, f)
                        ),
                    Err(_) => Ok(// continue
                        Some((lploc, l, lctx))
                    ),
                }
            }) {
            Ok(None) => {
                // left is empty, no overlap with this right
                None
            },
            Ok(Some((lploc, l, lctx))) => {
                // smallest didn't match
                // continue in smallest left
                let overlap =
                    self.find_overlap_with_right(
                        &l.sorted_patterns::<T, Left>(self),
                        right,
                    );
                overlap.map(|(found, _loc, p)|
                    (found, lploc.clone(), D::concat_inner_and_outer(p, lctx))
                )
            }
            Err((lploc, _l, lctx, found)) => {
                // some right matches
                // add larger patterns to right all
                let (overlap, _) = self.index_found(found);
                Some((overlap, lploc.clone(), lctx))
            }
        }
    }
    fn find_overlap_with_left(
        &mut self,
        mut rlarge: LocatedPatterns,
        left: Child,
        rps: LocatedPatterns,
    ) -> (Option<(Child, PatternLocation, Pattern)>, LocatedPatterns) {
        match rps.clone().into_iter()
            .enumerate()
            .try_fold(None, |_, (i, (rploc, rp))| {
                let (r, rctx) = Right::split_context_head(rp.clone()).unwrap();
                match self.find_parent([left, r].as_slice()) {
                    Ok(f) => Err(// break
                            (i, rploc, r, rctx, f)
                        ),
                    Err(_) => Ok(// continue
                        Some((rploc, r, rctx))
                    ),
                }
            }) {
            Ok(None) => {
                // right is empty, no overlap with this left
                (None, rlarge)
            },
            Ok(Some((rploc, r, rctx))) => {
                // smallest didn't match
                // continue in smallest right
                rlarge.extend(rps);
                let (overlap, rlarge) =
                    self.find_overlap_with_left(
                        rlarge,
                        left,
                        r.sorted_patterns::<T, Right>(self)
                    );
                (
                    overlap.map(|(found, _loc, p)|
                        (found, rploc, D::concat_inner_and_outer(p, rctx))
                    ),
                    rlarge,
                )
            }
            Err((i, rploc, _r, rctx, found)) => {
                // some right matches
                // add larger patterns to right all
                rlarge.extend(rps.into_iter().take(i));
                let (overlap, _) = self.index_found(found);
                (Some((overlap, rploc, rctx)), rlarge)
            }
        }
    }
    fn find_overlaps(
        &mut self,
        left: Child,
        right: Child,
    ) -> Overlaps {
        let lps = left.left_sorted_patterns(self);
        if let Some((overlap, lloc, lctx)) = self.find_overlap_with_right(&lps, right) {
            // try to find overlap with complete right
            vec![Overlap::EndPiece(lloc, lctx, overlap)]
        } else {
            let rps = right.right_sorted_patterns(self);
            let (_, overlaps) = lps.into_iter()
                .fold((rps, vec![]),
                |(rps, mut overlaps), (lloc, lp)| {
                let (l, lctx) = Left::split_context_head(lp).unwrap();
                let (overlap, larger)
                    = self.find_overlap_with_left(vec![], l, rps);

                if let Some((overlap, rloc, rctx)) = overlap {
                    overlaps.push(Overlap::Expandable(lloc, lctx, overlap, rloc, rctx));
                }
                // second iteration should use already fetched patterns
                (larger, overlaps)
            });
            overlaps
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct PatternGroup {
    //lead: Vec<Child>,
    index: Child,
    //overlaps: NewOverlaps, // overlaps of lead + postfix
}
impl PatternGroup {
    // - find overlaps with >lead postfix in previous PatternGroup
    // ||       |>        ||     *    |
    // ||   |       | | | ||
    // ||     |           ||
    //          |    |          |     |
    //          |  |     |            |
    // - try expanding overlaps in previous group
    // - create index for expandable right context
    // ||       |         ||          |
    // ||   |       | * |  *    |     | // expanded
    // ||     |           ||            // not expandable
    //          |  |      |            | // non expansion
    // - if any prev overlaps not expanded,
    //   create new index for prev group
    // - (include expanded overlaps before expansion too)
    // |         *         |          |
    // |    |       |   |       |     |
    //          |  |     |            |
    // - index left context in non expansion overlaps
    // |                   |          |
    // |    |       |   |       |     |
    //          |   *    |            |
    // - take prefix for any non expansion overlaps
    // |                   |          |
    // |    |       |   |       |     |
    //>|        |        |            |
    // - build new PatternGroup
    // ||                  |          ||
    // ||   |       |   |       |     ||
    // ||       |        |            || 
    //fn get_postfix(&self) -> Child {
    //    self.postfix.as_ref().map(|group| group.get_postfix())
    //        .unwrap_or_else(|| *self.lead.last().expect("empty PatternGroup"))
    //}
    //fn index_lead<'g, T: Tokenize, D: MatchDirection>(
    //    &self,
    //    reader: &mut Reader<'g, T, D>,
    //) -> Child {
    //    if self.postfix.is_some() {
    //        reader.graph.insert_pattern(self.lead.clone())    
    //    } else {
    //        let old_lead = self.lead.clone();
    //        let old_overlaps = self.overlaps.clone();
    //        let patterns = std::iter::once(old_lead).chain(
    //            old_overlaps.into_iter().map(|overlap| overlap.into_pattern())
    //        );
    //        reader.graph.insert_patterns(patterns)
    //    }
    //}
    //fn index<'g, T: Tokenize, D: MatchDirection>(
    //    mut self,
    //    reader: &mut Reader<'g, T, D>,
    //) -> Child {
    //    if let Some(postfix) = self.postfix.take() {
    //        self.lead.push(postfix.index(reader));
    //    }
    //    match self.lead.len() {
    //        0 => panic!("Empty PatternGroup"),
    //        1 => self.lead.into_iter().next().unwrap(),
    //        _ => {
    //            let patterns = std::iter::once(self.lead).chain(
    //                self.overlaps.into_iter().map(|overlap| overlap.into_pattern())
    //            );
    //            reader.graph.insert_patterns(patterns)
    //        }
    //    }
    //}
    //pub fn append<'g, T: Tokenize, D: MatchDirection>(
    //    &mut self,
    //    reader: &mut Reader<'g, T, D>,
    //    next: Child,
    //) {
    //    let post = self.get_postfix();
    //    let roverlaps = reader.find_overlaps(post.clone(), next)
    //        .into_iter()
    //        .map(|overlap| (overlap.get_left_ploc(), overlap))
    //        .collect::<OverlapMap>();
    //    if roverlaps.is_empty() {
    //        // if no new overlaps
    //        if self.overlaps.is_empty() {
    //            // if no old overlaps either
    //            // index and append current postfix if any
    //            if let Some(post) = self.postfix.take() {
    //                let post = post.index(reader);
    //                self.lead.push(post);
    //            }
    //            // append next
    //            self.lead.push(next);
    //        } else {
    //            // some old overlaps though
    //            let index = self.clone().index(reader);
    //            *self = Self::computed(vec![index, next], vec![]);
    //        }
    //    } else {
    //        // some new overlaps
    //        if !self.overlaps.is_empty() {
    //            // also some old overlaps
    //            let mut not_expandable = false;
    //            let mut new_overlaps = vec![];
    //            for loverlap in &mut self.overlaps {
    //                if let NewOverlap::Expandable(
    //                    llctx,
    //                    linner,
    //                    lrloc,
    //                    lrctx,
    //                ) = loverlap {
    //                    if let Some(roverlap) = roverlaps.get(lrloc) {
    //                        let mut ctx = lrctx.clone();
    //                        let ctxlen = ctx.len();
    //                        // pop last of right context to get inner context
    //                        ctx.pop();
    //                        let ctx = if let Some(ctx)
    //                            = reader.index_context(ctx, lrloc.clone().with_range(1..1 + ctxlen)) {
    //                            [llctx.as_slice(), &[linner.clone(), ctx]].concat()
    //                        } else {
    //                            // context empty
    //                            [llctx.as_slice(), &[linner.clone()]].concat()
    //                        };
    //                        new_overlaps.push(match roverlap {
    //                            Overlap::Expandable(
    //                                _,
    //                                _,
    //                                rinner,
    //                                rrloc,
    //                                rrctx
    //                            ) => NewOverlap::Expandable(
    //                                    ctx,
    //                                    rinner.clone(),
    //                                    rrloc.clone(),
    //                                    rrctx.clone()
    //                                ),
    //                            Overlap::EndPiece(
    //                                _,
    //                                _,
    //                                rinner,
    //                            ) => NewOverlap::EndPiece(
    //                                    ctx,
    //                                    rinner.clone(),
    //                                ),
    //                        });
    //                    } else {
    //                        not_expandable = true;
    //                    }
    //                } else {
    //                    not_expandable = true;
    //                }
    //            }
    //            // if any not expandable, add overlap with old pattern group
    //            if not_expandable {
    //                // any not expandable
    //                // index old lead to reuse as new lead
    //                let old_lead = self.index_lead(reader);
    //                // index old lead because we will use it in this index and in the next lead
    //                self.lead = vec![old_lead];
    //                // create new index for old pattern group
    //                let old = self.clone().index(reader);
    //                new_overlaps.push(NewOverlap::EndPiece(vec![old], next));
    //                // create new index for previous pattern group
    //            }
    //            // update overlaps with expanded
    //            self.overlaps = new_overlaps;
    //        }
    //        // new postfix with next and its overlaps
    //        // create new pattern group from remaining non expansion right overlaps
    //        let roverlaps = roverlaps.into_values()
    //            .map(Overlap::to_new_overlap)
    //            .collect();
    //        self.postfix = Some(Box::new(Self::computed(vec![post, next], roverlaps)));
    //    }
    //}
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum Overlap {
    Expandable(PatternLocation, Pattern, Child, PatternLocation, Pattern),
    EndPiece(PatternLocation, Pattern, Child),
}
impl Overlap {
    pub fn index<'g, T: Tokenize, D: IndexDirection>(&self, reader: &mut Reader<'g, T, D>) -> Pattern {
        match self {
            Self::Expandable(lloc, lctx, inner, rloc, rctx)
                => {
                let pre = reader.graph.index_range_at(lloc.clone().with_range(0..lctx.len())).unwrap();
                let post = reader.graph.index_range_at(rloc.clone().with_range(1..rctx.len()+1)).unwrap();
                vec![pre, inner.clone(), post]
            },
            Self::EndPiece(lloc, lctx, inner)
                => {
                let pre = reader.graph.index_range_at(lloc.clone().with_range(0..lctx.len())).unwrap();
                vec![pre, inner.clone()]
            }
        }
    }
}
type Overlaps = Vec<Overlap>;

// - has parent if patterns are stored in other index
// - patterns have id if they are stored in other index
type LocatedPatterns = Vec<(PatternLocation, Pattern)>;
trait PatternSource: Sized {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> LocatedPatterns;
    fn left_sorted_patterns<T: Tokenize>(self, graph: &Hypergraph<T>) -> LocatedPatterns {
        self.sorted_patterns::<T, Left>(graph)
    }
    fn right_sorted_patterns<T: Tokenize>(self, graph: &Hypergraph<T>) -> LocatedPatterns {
        self.sorted_patterns::<T, Right>(graph)
    }
}
impl PatternSource for Child {
    fn sorted_patterns<T: Tokenize, D: OverlapSide>(self, graph: &Hypergraph<T>) -> LocatedPatterns {
        if self.width > 1 {
            let patterns = self.vertex(graph)
                .get_children()
                .into_iter()
                .map(|(pid, p)| (PatternLocation::new(self.clone(), *pid), p.clone()))
                .collect();
            D::sort_by_inner(patterns)
        } else {
            vec![]
        }
    }
}
trait OverlapSide : MatchDirection {
    type Opposite: OverlapSide;
    /// sorts child patterns by decending width of their most inner child
    fn sort_by_inner(
        mut children: LocatedPatterns,
    ) -> LocatedPatterns {
        children.sort_unstable_by(|a, b|
            Self::get_ordering_element(&b.1).cmp(Self::get_ordering_element(&a.1))
        );
        children
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
        let x_id = xab.vertex(&graph)
            .find_child_pattern_id(|(_, p)| p.get(0).map(|c| *c == x).unwrap_or_default())
            .unwrap();
        let zw_id = yzw.vertex(&graph)
            .find_child_pattern_id(|(_, p)| p.get(0).map(|c| *c == yz).unwrap_or_default())
            .unwrap();
        let xa_id = xab.vertex(&graph)
            .find_child_pattern_id(|(_, p)| p.get(0).map(|c| *c == xa).unwrap_or_default())
            .unwrap();
        let w_id = yzw.vertex(&graph)
            .find_child_pattern_id(|(_, p)| p.get(1).map(|c| *c == w).unwrap_or_default())
            .unwrap();
        assert_eq!(
            overlaps,
            hashset![
                Overlap::Expandable(
                    PatternLocation {
                        parent: xab,
                        pattern_id: x_id,
                    },
                    vec![x],
                    aby,
                    PatternLocation {
                        parent: yzw,
                        pattern_id: zw_id,
                    },
                    vec![z, w]
                ),
                Overlap::Expandable(
                    PatternLocation {
                        parent: xab,
                        pattern_id: xa_id,
                    },
                    vec![xa],
                    byz,
                    PatternLocation {
                        parent: yzw,
                        pattern_id: w_id,
                    },
                    vec![w]
                ),
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
        let overlaps: HashSet<_> = graph.right_reader().read_overlaps(vis, u).into_iter().collect();
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
                vec![
                    vi,
                    su,
                ],
            ]
        )
    }
}