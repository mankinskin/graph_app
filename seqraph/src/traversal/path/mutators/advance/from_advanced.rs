use crate::*;

pub trait FromAdvanced<A: Advanced> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: A, trav: &'a Trav) -> Self;
}
impl FromAdvanced<SearchPath> for FoundPath {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: SearchPath, trav: &'a Trav) -> Self {
        if path.is_complete::<_, D, _>(trav) {
            Self::Complete(<SearchPath as GraphRootChild<Start>>::graph_root_child_location(&path).parent)
        } else {
            Self::Range(path)
        }
        
    }
}
impl FromAdvanced<OriginPath<SearchPath>> for OriginPath<FoundPath> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: OriginPath<SearchPath>, trav: &'a Trav) -> Self {
        Self {
            postfix: FoundPath::from_advanced::<_, D, _>(path.postfix, trav),
            origin: path.origin,
        }
    }
}

impl<A: Advanced, F: FromAdvanced<A>> FromAdvanced<A> for OriginPath<F> {
    fn from_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(path: A, trav: &'a Trav) -> Self {
        Self {
            origin: MatchEnd::Path(HasRootedPath::<Start>::child_path(&path).clone()),
            postfix: F::from_advanced::<_, D, _>(path, trav),
        }
    }
}