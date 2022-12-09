use crate::{
    r#match::*,
    Hypergraph,
};
use itertools::*;

#[derive(Clone, Debug)]
pub struct Matcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'g, T: Tokenize, D: MatchDirection> std::ops::Deref for Matcher<'g, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Matcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    /// match a query with the context of a child at a path
    pub(crate) fn match_path_in_context(
        &self,
        mut path: Vec<ChildLocation>,
        query: impl IntoPattern<Item = impl AsChild>,
    ) -> Result<GrownPath, MismatchPath> {
        let query = query.into_pattern();
        path.pop().map(|mut location| {
            // start index is some descendant of parent, with location
            let vertex = self.expect_vertex_data(location.parent);
            let child_patterns = vertex.get_children();
            let pattern = child_patterns.get(&location.pattern_id).unwrap();
            let child_tail = D::front_context(pattern, location.sub_index);
            if let Some((skipped, eob)) = D::skip_equal_indices(child_tail.iter(), query.iter()) {
                match eob {
                    // different elements on both sides
                    EitherOrBoth::Both(&child_next, &query_next) => {
                        // select larger and smaller
                        if child_next.width <= query_next.width {
                            // relatives can not have same sizes or
                            // next query is larger than next child, mismatch
                            location.sub_index += skipped;
                            path.push(location);
                            Err(MismatchPath {
                                path,
                                child: D::split_end_normalized(child_tail.as_pattern_view(), skipped),
                                query: D::split_end_normalized(query.as_pattern_view(), skipped),
                            })
                        } else {
                            // next child is larger than next query, search next query in prefix of next child
                            let query_context = D::front_context_normalized(query.as_pattern_view(), skipped);
                            //let child_context = D::front_context_normalized(child_tail.as_pattern_view(), skipped);
                            let child_next = child_next.as_child();
                            let query_next = query_next.as_child();
                            // breadth first find query_next in child_next
                            location.sub_index += skipped;
                            let mut loc = location.clone();
                            loc.sub_index += 1;
                            Bft::new((vec![loc], child_next), |(path, parent)| {
                                let vertex = self.expect_vertex_data(&parent);
                                let child_patterns = vertex.get_children();
                                child_patterns
                                    .into_iter()
                                    .filter_map(|(&pid, pattern)| {
                                        let &head = D::pattern_head(pattern).unwrap();
                                        if head.width() >= query_next.width() {
                                            let mut path = path.clone();
                                            let next_index = D::head_index(pattern);
                                            path.push(ChildLocation::new(parent.clone(), pid, next_index));
                                            Some((path, head))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect_vec()
                                    .into_iter()
                            })
                            // find child starting with next_child
                            .find_map(|(_, (path, node))|
                                (node == query_next).then(||
                                    Ok(path)
                                )
                            )
                            // none found
                            .unwrap_or_else(|| {
                                path.push(location);
                                Err(MismatchPath {
                                    // todo add path
                                    path: path.clone(),
                                    child: D::split_end_normalized(child_tail.as_pattern_view(), skipped),
                                    query: D::split_end_normalized(query.as_pattern_view(), skipped),
                                })
                            })
                            // found path to query_next in child_next
                            .and_then(|inner_path| {
                                path.extend(inner_path);
                                self.match_path_in_context(path, query_context)
                            })
                        }
                    }
                    EitherOrBoth::Right(_) => {
                        // child tail ended, so the remaining path points to a matching index
                        let query_context = D::split_end_normalized(&query, skipped);
                        self.match_path_in_context(path, query_context)
                    },
                    EitherOrBoth::Left(_) => {
                        // query ended
                        location.sub_index += skipped;
                        path.push(location);
                        Ok(GrownPath {
                            path,
                            remainder: GrowthRemainder::Child(D::split_end_normalized(&child_tail, skipped)),
                        })
                    },
                }
            } else {
                self.match_path_in_context(path, vec![] as Pattern)
            }
        })
        .unwrap_or_else(|| {
            Ok(GrownPath {
                path: vec![],
                remainder: if query.is_empty() {
                    GrowthRemainder::None
                } else {
                    GrowthRemainder::Query(query)
                },
            })
        })
    }
}
