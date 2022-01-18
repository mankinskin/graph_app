use crate::{
    r#match::*,
    vertex::*,
    *,
};
mod searcher;
pub use searcher::*;
//mod async_searcher;
//pub use async_searcher::*;
mod bft;
pub use bft::*;

// TODO: rename to something with context
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParentMatch {
    pub parent_range: FoundRange, // context in parent
    pub remainder: Option<Pattern>, // remainder of query
}
impl ParentMatch {
    pub fn embed_in_super(
        self,
        other: Self,
    ) -> Self {
        Self {
            parent_range: self.parent_range.embed_in_super(other.parent_range),
            remainder: other.remainder,
        }
    }
}
pub type ParentMatchResult = Result<ParentMatch, NoMatch>;

// found range of search pattern in vertex at index
// TODO: actually a context
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum FoundRange {
    Complete,                // Full index found
    Prefix(Pattern),         // found prefix (remainder)
    Postfix(Pattern),        // found postfix (remainder)
    Infix(Pattern, Pattern), // found infix
}
impl FoundRange {
    pub fn matches_completely(&self) -> bool {
        self == &FoundRange::Complete
    }
    pub fn reverse(self) -> Self {
        match self {
            Self::Complete => Self::Complete,
            Self::Prefix(post) => Self::Postfix(post),
            Self::Postfix(pre) => Self::Prefix(pre),
            Self::Infix(pre, post) => Self::Infix(post, pre),
        }
    }
    pub fn embed_in_super(
        self,
        other: Self,
    ) -> Self {
        match (self, other) {
            (Self::Complete, outer) => outer,
            (inner, Self::Complete) => inner,
            (Self::Prefix(inner), Self::Postfix(outer)) => Self::Infix(outer, inner),
            (Self::Prefix(inner), Self::Prefix(outer)) => Self::Prefix([inner, outer].concat()),
            (Self::Prefix(inner), Self::Infix(louter, router)) => {
                Self::Infix(louter, [inner, router].concat())
            }
            (Self::Postfix(inner), Self::Prefix(outer)) => Self::Infix(inner, outer),
            (Self::Postfix(inner), Self::Postfix(outer)) => Self::Postfix([outer, inner].concat()),
            (Self::Postfix(inner), Self::Infix(louter, router)) => {
                Self::Infix([louter, inner].concat(), router)
            }
            (Self::Infix(linner, rinner), Self::Prefix(outer)) => {
                Self::Infix(linner, [rinner, outer].concat())
            }
            (Self::Infix(linner, rinner), Self::Postfix(outer)) => {
                Self::Infix([outer, linner].concat(), rinner)
            }
            (Self::Infix(linner, rinner), Self::Infix(louter, router)) => {
                Self::Infix([louter, linner].concat(), [rinner, router].concat())
            }
        }
    }
}

//#[derive(Debug, PartialEq, Eq, Clone, Hash)]
//pub struct SearchFound {
//    pub parent_match: ParentMatch, // match ranges
//    //pub index: Child, // index in which we found the query
//    //pub pattern_id: PatternId, // pattern
//    //pub sub_index: usize, // starting index in pattern
//    pub location: PatternRangeLocation,
//    pub path: Vec<Child>,
//}
//impl SearchFound {
//    pub fn unwrap_complete(self) -> Child {
//        if let FoundRange::Complete = self.parent_match.parent_range {
//            self.location.parent
//        } else {
//            panic!("Failed to unwrap {:#?} as complete match!", self);
//        }
//    }
//    pub fn expect_complete(self, msg: &str) -> Child {
//        if let FoundRange::Complete = self.parent_match.parent_range {
//            self.location.parent
//        } else {
//            panic!("Failed to unwrap {:#?} as complete match: {}", self, msg);
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundPath {
    root: Child,
    start_path: ChildPath,
    end_path: ChildPath,
    remainder: Option<Pattern>
}
impl FoundPath {
    pub fn complete(child: impl AsChild) -> Self {
        Self {
            root: child.as_child(),
            start_path: vec![],
            end_path: vec![],
            remainder: None,
        }
    }
    pub fn found_complete(&self) -> bool {
        self.start_path.is_empty()
            && self.end_path.is_empty()
            && self.remainder.is_none()
    }
    pub fn unwrap_complete(self) -> Child {
        if self.found_complete() {
            self.root
        } else {
            panic!("Failed to unwrap {:#?} as complete match!", self);
        }
    }
    pub fn expect_complete(self, msg: &str) -> Child {
        if self.found_complete() {
            self.root
        } else {
            panic!("Failed to unwrap {:#?} as complete match: {}", self, msg);
        }
    }
}
pub type SearchResult = Result<FoundPath, NoMatch>;

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub(crate) fn right_searcher(&'a self) -> Searcher<'a, T, Right> {
        Searcher::new(self)
    }
    #[allow(unused)]
    pub(crate) fn left_searcher(&'a self) -> Searcher<'a, T, Left> {
        Searcher::new(self)
    }
    pub(crate) fn find_ancestor(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.to_children(pattern);
        self.right_searcher().find_pattern_ancestor(pattern)
    }
    pub(crate) fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.to_children(pattern);
        self.right_searcher().find_pattern_parent(pattern)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.to_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}
#[macro_use]
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        graph::tests::context,
        Child,
        r#match::*,
    };
    use pretty_assertions::{
        assert_eq,
        assert_matches,
    };
    use itertools::*;

    #[test]
    fn find_ancestor1() {
        let (
            graph,
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h,
            i,
            ab,
            bc,
            _cd,
            _bcd,
            abc,
            abcd,
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            ababababcdefghi,
        ) = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_eq!(
            graph.find_ancestor(&bc_pattern),
            Ok(FoundPath {
                root: *bc,
                remainder: None,
                start_path: vec![],
                end_path: vec![],
            }),
            "bc"
        );
        assert_eq!(
            graph.find_ancestor(&b_c_pattern),
            Ok(FoundPath::complete(bc)),
            "b_c"
        );
        assert_eq!(
            graph.find_ancestor(&a_bc_pattern),
            Ok(FoundPath::complete(abc)),
            "a_bc"
        );
        assert_eq!(
            graph.find_ancestor(&ab_c_pattern),
            Ok(FoundPath::complete(abc)),
            "ab_c"
        );
        assert_eq!(
            graph.find_ancestor(&a_bc_d_pattern),
            Ok(FoundPath::complete(abcd)),
            "a_bc_d"
        );
        assert_eq!(
            graph.find_ancestor(&a_b_c_pattern),
            Ok(FoundPath::complete(abc)),
            "a_b_c"
        );
        let a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern =
            vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
        assert_eq!(
            graph.find_ancestor(&a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern),
            Ok(FoundPath::complete(ababababcdefghi)),
            "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
        );
        let a_b_c_c_pattern = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
        assert_eq!(
            graph.find_ancestor(&a_b_c_c_pattern),
            Ok(FoundPath {
                root: *abc,
                end_path: vec![],
                start_path: vec![],
                remainder: Some(vec![*c]),
            }),
            "a_b_c_c"
        );
    }
    #[test]
    fn find_ancestor2() {
        let mut graph = Hypergraph::default();
        let (a, b, _w, x, y, z) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ]).into_iter().next_tuple().unwrap();
        let ab = graph.insert_pattern([a, b]);
        let by = graph.insert_pattern([b, y]);
        let yz = graph.insert_pattern([y, z]);
        let xa = graph.insert_pattern([x, a]);
        let xab = graph.insert_patterns([[x, ab], [xa, b]]);
        let (xaby, xaby_ids) = graph.insert_patterns_with_ids([vec![xab, y], vec![xa, by]]);
        let xa_by_id = xaby_ids[1];
        let (xabyz, xabyz_ids) = graph.insert_patterns_with_ids([vec![xaby, z], vec![xab, yz]]);
        let xaby_z_id = xabyz_ids[0];
        let byz_found = graph.find_ancestor(vec![by, z]);
        assert_eq!(
            byz_found,
            Ok(FoundPath {
                root: xabyz,
                remainder: None,
                start_path: vec![
                    ChildLocation {
                        parent: xaby,
                        pattern_id: xa_by_id,
                        sub_index: 1,
                    },
                ],
                end_path: vec![],
            })
        );
    }
    //#[test]
    //fn find_sequence() {
    //    let (
    //        graph,
    //        _a,
    //        _b,
    //        _c,
    //        _d,
    //        _e,
    //        _f,
    //        _g,
    //        _h,
    //        _i,
    //        _ab,
    //        _bc,
    //        _cd,
    //        _bcd,
    //        abc,
    //        _abcd,
    //        _ef,
    //        _cdef,
    //        _efghi,
    //        _abab,
    //        _ababab,
    //        ababababcdefghi,
    //    ) = &*context();
    //    assert_match!(
    //        graph.find_sequence("a".chars()),
    //        Err(NoMatch::SingleIndex),
    //        "a"
    //    );

    //    let abc_found = graph.find_sequence("abc".chars());
    //    assert_matches!(
    //        abc_found,
    //        Ok(SearchFound {
    //            parent_match: ParentMatch {
    //                parent_range: FoundRange::Complete,
    //                ..
    //            },
    //            ..
    //        })
    //    );
    //    assert_eq!(abc_found.unwrap().location.parent, *abc);
    //    let ababababcdefghi_found = graph.find_sequence("ababababcdefghi".chars());
    //    assert_matches!(
    //        ababababcdefghi_found,
    //        Ok(SearchFound {
    //            parent_match: ParentMatch {
    //                parent_range: FoundRange::Complete,
    //                ..
    //            },
    //            ..
    //        })
    //    );
    //    assert_eq!(ababababcdefghi_found.unwrap().location.parent, *ababababcdefghi);
    //}
}
