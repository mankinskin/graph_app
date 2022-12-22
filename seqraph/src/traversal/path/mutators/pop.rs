use crate::*;

pub trait PathPop: Send + Sync {
    type Result;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav) -> Self::Result;
}

impl PathPop for OriginPath<SearchPath> {
    type Result = OriginPath<<SearchPath as PathPop>::Result>;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav) -> Self::Result {
        OriginPath {
            postfix: self.postfix.pop_path::<_, D, _>(trav),
            origin: self.origin.pop_path::<_, D, _>(trav)
                .unwrap_or_else(|err| MatchEnd::Complete(err))
        }
    }
}

impl PathPop for SearchPath {
    type Result = <ChildPath as PathPop>::Result;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav) -> Self::Result {
        self.start.pop_path::<_, D, _>(trav)
    }
}

impl PathPop for ChildPath {
    type Result = MatchEnd<ChildPath>;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav) -> Self::Result {
        match self {
            ChildPath::Leaf(leaf) => MatchEnd::Complete(leaf.child),
            ChildPath::Path { entry, mut path, child, width, token_pos } => {
                MatchEnd::Path(if let Some(seg) = path.pop() {
                    if path.is_empty() {
                        ChildPath::Leaf(PathLeaf {
                            entry: seg,
                            child,
                            width,
                            token_pos,
                        })
                    } else {
                        ChildPath::Path {
                            entry: seg,
                            path,
                            width,
                            child,
                            token_pos,
                        }
                    }
                } else {
                    let graph = trav.graph();
                    ChildPath::Leaf(PathLeaf {
                        child: graph.expect_child_at(&entry),
                        entry,
                        width,
                        token_pos,
                    })
                })
            },
        }
    }
}