use super::*;

#[async_trait]
pub(crate) trait PathComplete: Send + Sync {
    //fn new_complete(c: Child) -> Self;
    async fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child>;

    async fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        self.complete::<_, D, _>(trav).await.is_some()
    }
}

#[async_trait]
impl<P: PathComplete> PathComplete for OriginPath<P> {
    async fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.postfix.complete::<_, D, _>(trav).await
    }
}
#[async_trait]
impl PathComplete for SearchPath {
    async fn is_complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        let pattern = self.get_entry_pattern(trav).await;
        <StartPath as PathBorder<D>>::pattern_is_complete(self.start_match_path(), &pattern[..]) &&
            self.end_path().is_empty() &&
            <EndPath as PathBorder<D>>::pattern_entry_outer_pos(pattern, self.get_exit_pos()).is_none()
    }
    async fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        self.is_complete::<_, D, _>(trav).await.then(||
            self.root_child()
        )
    }
}
#[async_trait]
impl PathComplete for StartLeaf {
    /// returns child if reduced to single child
    async fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        let graph = trav.graph().await;
        let pattern = graph.expect_pattern_at(self.entry);
        (self.entry.sub_index == D::head_index(pattern.borrow()))
            .then(|| self.entry.parent)
    }
}
#[async_trait]
impl PathComplete for StartPath {
    /// returns child if reduced to single child
    async fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        match self {
            Self::Leaf(leaf) => leaf.complete::<_, D, _>(trav).await,
            // TODO: maybe skip path segments starting at pattern head
            Self::Path { .. } => None,
        }
    }
}