use crate::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct PrefixQuery {
    pub pattern: Pattern,
    pub exit: usize,
    pub end: RolePath<End>,
    pub width: usize,
}

impl PrefixQuery {
    pub fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(pattern: P) -> Result<Self, NoMatch> {
        let exit = D::head_index(pattern.borrow());
        let pattern = pattern.into_pattern();
        match pattern.len() {
            0 => Err(NoMatch::EmptyPatterns),
            1 => Err(NoMatch::SingleIndex(pattern.into_iter().next().unwrap())),
            _ => Ok(Self {
                    pattern,
                    exit,
                    width: 0,
                    end: None,
                })
        }
    }
}
//impl HasRolePath for PrefixQuery {
//    fn role_path(&self) -> &[ChildLocation] {
//        self.end.borrow()
//    }
//}


//impl TraversalPath for PrefixQuery {
//    fn prev_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: Trav) -> Option<usize> {
//        if self.end.is_empty() {
//            D::pattern_index_prev(self.pattern.borrow(), self.exit)
//        } else {
//            let location = self.end.last().unwrap();
//            let pattern = trav.graph().expect_pattern_at(location);
//            D::pattern_index_prev(pattern, location.sub_index)
//        }
//    }
//}
impl Wide for PrefixQuery {
    fn width(&self) -> usize {
        self.width
    }
}
impl WideMut for PrefixQuery {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}


#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use super::PrefixQuery;
    use crate::{index::Right, Hypergraph, Token, traversal::Advance};
    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    #[test]
    fn prefix_path_reconstruct1() {
        let mut graph = Hypergraph::new();
        let (a, b, c, d, e, f, g) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
            Token::Element('e'),
            Token::Element('f'),
            Token::Element('g'),
        ]).into_iter().next_tuple().unwrap();

        let pattern = vec![c,d,d,e,f,g,a,c,d,e,f,a,g,f,g,g,e,d,b,d];
        let mut path = PrefixQuery::new_directed::<Right, _>(pattern.borrow()).unwrap();
        let mut rec = vec![];
        while let Some(next) = path.advance::<_, Right, _>(&graph) {
            rec.push(next);
        }
        assert_eq!(rec, pattern);
    }
}