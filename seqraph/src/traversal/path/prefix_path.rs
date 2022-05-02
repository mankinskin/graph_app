use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone)]
pub struct PrefixPath {
    pub(crate) pattern: Pattern,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl<
    'a: 'g,
    'g,
> PrefixPath {
    pub fn new_directed<
        D: MatchDirection + 'a,
        P: IntoPattern,
    >(pattern: P) -> Result<Self, NoMatch> {
        let exit = D::head_index(pattern.borrow());
        let pattern = pattern.into_pattern();
        match pattern.len() {
            0 => Err(NoMatch::EmptyPatterns),
            1 => Err(NoMatch::SingleIndex),
            _ => 
            Ok(Self {
                pattern,
                exit,
                end: vec![],
            })
        }
    }
}