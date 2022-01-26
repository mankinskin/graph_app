use crate::{
    vertex::*,
    r#match::*,
    index::*,
    Hypergraph,
};

#[derive(Debug)]
pub struct Indexer<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'a, T: Tokenize> std::ops::Deref for Indexer<'a, T> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize> std::ops::DerefMut for Indexer<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g> Indexer<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    pub(crate) fn index_found(
        &mut self,
        found_path: FoundPath,
    ) -> (Option<Child>, Child, Option<Child>, Pattern) {
        let FoundPath {
                root,
                start_path,
                end_path,
                remainder,
        } = found_path;
        println!("start: {:?}, end: {:?}", start_path.as_ref().map(|p| p.last().unwrap()), end_path.as_ref().map(|p| p.last().unwrap()));
        let left = start_path.map(|start_path| {
            let mut start_path = start_path.into_iter();
            let location = start_path.next().unwrap();
            let inner = self.index_postfix_at(&location).unwrap();
            start_path
                .fold((None, inner, location), |(context, inner, prev_location), location| {
                    let context = context.unwrap_or_else(||
                        self.index_pre_context_at(&prev_location).unwrap()
                    );
                    let context = self.index_pre_context_at(&location).map(|pre|
                            self.insert_pattern([pre, context])
                        )
                        .unwrap_or(context);
                    let inner = self.index_post_context_at(&location).map(|postfix|
                        self.insert_pattern([inner, postfix])
                    ).unwrap_or(inner);
                    self.add_pattern_to_node(location.parent, [context, inner].as_slice());
                    (Some(context), inner, location)
                })
        });
        let right = end_path.map(|end_path| {
            let mut end_path = end_path.into_iter().rev();
            let location = end_path.next().unwrap();
            let inner = self.index_prefix_at(&location).unwrap();
            end_path
                .fold((inner, None, location), |(inner, context, prev_location), location| {
                    let context = context.unwrap_or_else(||
                        self.index_post_context_at(&prev_location).unwrap()
                    );
                    let context = self.index_post_context_at(&location).map(|post|
                            self.insert_pattern([context, post])
                        )
                        .unwrap_or(context);
                    let inner = self.index_pre_context_at(&location).map(|pre|
                        self.insert_pattern([pre, inner])
                    ).unwrap_or(inner);
                    self.add_pattern_to_node(location.parent, [inner, context].as_slice());
                    (inner, Some(context), location)
                })
        });
        let (lctx, inner, rctx) = match (left, right) {
            (None, None) => (None, root, None),
            (Some((lcontext, linner, _)), Some((rinner, rcontext, _))) => {
                let inner = self.insert_pattern([linner, rinner].as_slice());
                match (lcontext, rcontext) {
                    (Some(lctx), Some(rctx)) => {
                        self.add_pattern_to_node(root, [lctx, inner, rctx].as_slice());
                    }
                    (Some(lctx), None) => {
                        self.add_pattern_to_node(root, [lctx, inner].as_slice());
                    }
                    (None, Some(rctx)) => {
                        self.add_pattern_to_node(root, [inner, rctx].as_slice());
                    }
                    (None, None) => unreachable!(),
                };
                (lcontext, inner, rcontext)
            },
            (Some((lcontext, inner, _)), None) => {
                (lcontext, inner, None)
            }
            (None, Some((inner, rcontext, _))) => {
                (None, inner, rcontext)
            }
        };
        (lctx, inner, rctx, remainder.unwrap_or_default())
    }
    /// includes location
    pub(crate) fn index_prefix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.index_range_in(location.parent, location.pattern_id, 0..location.sub_index + 1)
    }
    /// includes location
    pub(crate) fn index_postfix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.index_range_in(location.parent, location.pattern_id, location.sub_index..)
    }
    /// does not include location
    pub(crate) fn index_pre_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.index_range_in(location.parent, location.pattern_id, 0..location.sub_index)
    }
    /// does not include location
    pub(crate) fn index_post_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.index_range_in(location.parent, location.pattern_id, location.sub_index + 1..)
    }
}