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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundPath {
    pub(crate) root: Child,
    pub(crate) start_path: Option<ChildPath>,
    pub(crate) end_path: Option<ChildPath>,
    pub(crate) remainder: Option<Pattern>
}
impl FoundPath {
    pub fn complete(root: impl AsChild) -> Self {
        Self {
            root: root.as_child(),
            start_path: None,
            end_path: None,
            remainder: None,
        }
    }
    pub fn remainder(root: impl AsChild, remainder: impl IntoPattern<Item=impl AsChild>) -> Self {
        Self {
            root: root.as_child(),
            start_path: None,
            end_path: None,
            remainder: if remainder.is_empty() {
                None
            } else {
                Some(remainder.into_pattern())
            },
        }
    }
    pub fn new(root: impl AsChild, start_path: ChildPath, end_path: ChildPath, remainder: impl IntoPattern<Item=impl AsChild>) -> Self {
        Self {
            root: root.as_child(),
            start_path: if start_path.is_empty() {
                None
            } else {
                Some(start_path)
            },
            end_path: if end_path.is_empty() {
                None
            } else {
                Some(end_path)
            },
            remainder: if remainder.is_empty() {
                None
            } else {
                Some(remainder.into_pattern())
            },
        }
    }
    pub fn no_remainder(root: impl AsChild, start_path: ChildPath, end_path: ChildPath) -> Self {
        Self {
            root: root.as_child(),
            start_path: if start_path.is_empty() {
                None
            } else {
                Some(start_path)
            },
            end_path: if end_path.is_empty() {
                None
            } else {
                Some(end_path)
            },
            remainder: None,
        }
    }
    pub fn found_complete(&self) -> bool {
        self.start_path.is_none()
            && self.end_path.is_none()
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
    pub(crate) fn find_ancestor_in_context(
        &self,
        head: impl AsChild,
        context: impl IntoPattern<Item = impl AsChild>,
    ) -> SearchResult {
        self.right_searcher().find_ancestor_in_context(head, context)
    }
    #[allow(unused)]
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
    };
    use pretty_assertions::{
        assert_eq,
    };
    use itertools::*;

    #[test]
    fn find_ancestor1() {
        let Context {
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
            abc,
            abcd,
            ababababcdefghi,
            ..
         } = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_eq!(
            graph.find_ancestor(&bc_pattern),
            Err(NoMatch::SingleIndex),
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
            Ok(FoundPath::remainder(abc, [c].as_slice())),
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
        let _xaby_z_id = xabyz_ids[0];
        let byz_found = graph.find_ancestor(vec![by, z]);
        assert_eq!(
            byz_found,
            Ok(FoundPath {
                root: xabyz,
                remainder: None,
                start_path: Some(vec![
                    ChildLocation {
                        parent: xaby,
                        pattern_id: xa_by_id,
                        sub_index: 1,
                    },
                ]),
                end_path: None,
            })
        );
    }
    #[test]
    fn find_sequence() {
        let Context {
            graph,
            abc,
            ababababcdefghi,
            ..
         } = &*context();
        assert_eq!(
            graph.find_sequence("a".chars()),
            //Ok(FoundPath::complete(a))
            Err(NoMatch::SingleIndex),
        );

        let abc_found = graph.find_sequence("abc".chars());
        assert_eq!(
            abc_found,
            Ok(FoundPath::complete(abc))
        );
        let ababababcdefghi_found = graph.find_sequence("ababababcdefghi".chars());
        assert_eq!(
            ababababcdefghi_found,
            Ok(FoundPath::complete(ababababcdefghi))
        );
    }
}
