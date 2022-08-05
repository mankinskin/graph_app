use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct PrefixPath {
    pub(crate) pattern: Pattern,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
    pub(crate) width: usize,
    pub(crate) finished: bool,
}

impl<
    'a: 'g,
    'g,
> PrefixPath {
    pub fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(pattern: P) -> Result<Self, NoMatch> {
        let exit = D::head_index(pattern.borrow());
        let pattern = pattern.into_pattern();
        match pattern.len() {
            0 => Err(NoMatch::EmptyPatterns),
            1 => Err(NoMatch::SingleIndex),
            _ => Ok(Self {
                    pattern,
                    exit,
                    width: 0,
                    end: vec![],
                    finished: false,
                })
        }
    }
}
impl EntryPos for PrefixPath {
    fn get_entry_pos(&self) -> usize {
        0
    }
}
impl PatternEntry for PrefixPath {
    fn get_entry_pattern(&self) -> &[Child] {
        self.pattern.borrow()
    }
}
impl HasStartPath for PrefixPath {
    fn start_path(&self) -> &[ChildLocation] {
        &[]
    }
}
impl PatternStart for PrefixPath {}
impl EndPathMut for PrefixPath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
impl ExitPos for PrefixPath {
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
}
impl ExitMut for PrefixPath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl PatternExit for PrefixPath {
    fn get_exit_pattern(&self) -> &[Child] {
        &self.pattern
    }
}
impl HasEndPath for PrefixPath {
    fn end_path(&self) -> &[ChildLocation] {
        self.end.borrow()
    }
}
impl PatternEnd for PrefixPath {}

impl PathFinished for PrefixPath {
    fn is_finished(&self) -> bool {
        self.finished
    }
    fn set_finished(&mut self) {
        self.finished = true;
    }
}
impl End for PrefixPath {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        self.get_pattern_end(trav)
    }
}
impl ReduciblePath for PrefixPath {
    fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        if self.end.is_empty() {
            D::pattern_index_prev(self.pattern.borrow(), self.exit)
        } else {
            let location = self.end.last().unwrap();
            let pattern = trav.graph().expect_pattern_at(location);
            D::pattern_index_prev(pattern, location.sub_index)
        }
    }
}
impl Wide for PrefixPath {
    fn width(&self) -> usize {
        self.width
    }
}
impl WideMut for PrefixPath {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}
impl AdvanceablePath for PrefixPath {}


#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use super::PrefixPath;
    use crate::{index::Right, Hypergraph, Token, traversal::AdvanceablePath};
    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    #[test]
    fn prefix_path_reconstruct1() {
        let mut graph = Hypergraph::new();
        let (a, b, c, d, e, f, g) = graph.index_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
            Token::Element('e'),
            Token::Element('f'),
            Token::Element('g'),
        ]).into_iter().next_tuple().unwrap();

        let pattern = vec![c,d,d,e,f,g,a,c,d,e,f,a,g,f,g,g,e,d,b,d];
        let mut path = PrefixPath::new_directed::<Right, _>(pattern.borrow()).unwrap();
        let mut rec = vec![];
        loop {
            match path.try_get_advance::<_, Right, _>(&graph) {
                Ok((next, adv)) => {
                    path = adv;
                    rec.push(next);
                    continue
                },
                Err(next) => {
                    rec.push(next);
                    break
                }
            }
        }
        assert_eq!(rec, pattern);
    }
}