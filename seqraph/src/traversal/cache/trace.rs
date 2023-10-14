use crate::*;

pub trait Trace {
    fn trace<Trav: Traversable>(&self, trav: &Trav, cache: &mut TraversalCache);
}
impl Trace for EndState {
    fn trace<Trav: Traversable>(
        &self,
        trav: &Trav,
        cache: &mut TraversalCache,
    ) {
        match &self.kind {
            EndKind::Range(p) => {
                let root_entry = p.path
                    .role_root_child_location::<Start>()
                    .sub_index;
                cache.trace_path(
                    trav,
                    root_entry,
                    &p.path,
                    self.root_pos,
                    true,
                )
            },
            EndKind::Prefix(p) =>
                cache.trace_path(
                    trav,
                    0,
                    &p.path,
                    self.root_pos,
                    true,
                ),
            _ => {}
        }
    }
}
//impl Trace for ChildState {
//    fn trace<Trav: Traversable>(&self, trav: &Trav, cache: &mut TraversalCache) {
//        cache.trace_path(
//            trav,
//            self.paths.path.role_root_child_location::<Start>().sub_index,
//            &self.paths.path,
//            self.root_pos,
//            false,
//        );
//    }
//}

#[cfg(test)]
pub(crate) mod tests {
    use crate::*;
    use pretty_assertions::assert_eq;

    pub fn build_trace1() -> FoldState {
        let Context {
            graph,
            a,
            d,
            e,
            bc,
            ..
        } = &*context_mut();
        let query = vec![*a, *bc, *d, *e];
        graph.searcher().find_pattern_ancestor(query)
            .unwrap().result
            .unwrap_incomplete()
    }
    #[test]
    fn trace_graph1() {
        let res = build_trace1();
        let Context {
            a,
            e,
            abc,
            abcd,
            abc_d_id,
            abcdef,
            //abc_def_id,
            abcd_ef_id,
            //def,
            ef,
            e_f_id,
            ..
        } = &*context_mut();

        assert_eq!(res.start, *a);
        assert_eq!(res.end_state.width(), 5);

        assert_eq!(
            res.cache.entries[&lab!(a)], 
            VertexCache {
                index: *a,
                bottom_up: HashMap::from_iter([]),
                top_down: HashMap::from_iter([]),
            },
        );
        assert_eq!(
            res.cache.entries[&lab!(abcd)], 
            VertexCache {
                index: *abcd,
                bottom_up: HashMap::from_iter([
                    (3.into(), PositionCache {
                        edges: Edges {
                            top: Default::default(),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::up(*abc, 1),
                                    SubLocation::new(*abc_d_id, 0),
                                )
                            ]),
                        },
                        index: *abcd,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([]),
            }
        );
        assert_eq!(
            res.cache.entries[&lab!(ef)], 
            VertexCache {
                index: *ef,
                bottom_up: HashMap::from_iter([]),
                top_down: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Edges {
                            top: HashSet::from_iter([]),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::down(*e, 4),
                                    SubLocation::new(*e_f_id, 0),
                                )
                            ]),
                        },
                        index: *ef,
                        waiting: Default::default(),
                    })
                ]),
            }
        );
        assert_eq!(
            res.cache.entries[&lab!(e)], 
            VertexCache {
                index: *e,
                top_down: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Default::default(),
                        index: *e,
                        waiting: Default::default(),
                    })
                ]),
                bottom_up: HashMap::from_iter([]),
            },
        );
        assert_eq!(
            res.cache.entries[&lab!(abc)], 
            VertexCache {
                index: *abc,
                bottom_up: HashMap::from_iter([
                    (1.into(), PositionCache {
                        edges: Edges {
                            top: Default::default(),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::up(*a, 0),
                                    SubLocation::new(3, 0),
                                )
                            ]),
                        },
                        index: *abc,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([]),
            }
        );
        assert_eq!(
            res.cache.entries[&lab!(abcdef)], 
            VertexCache {
                index: *abcdef,
                bottom_up: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Edges {
                            top: HashSet::from_iter([]),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::up(*abcd, 3),
                                    SubLocation::new(*abcd_ef_id, 0),
                                ),
                            ]),
                        },
                        index: *abcdef,
                        waiting: Default::default(),
                    })
                ]),
                top_down: HashMap::from_iter([
                    (4.into(), PositionCache {
                        edges: Edges {
                            top: HashSet::from_iter([]),
                            bottom: HashMap::from_iter([
                                (
                                    DirectedKey::down(*ef, 4),
                                    SubLocation::new(*abcd_ef_id, 1),
                                )
                            ]),
                        },
                        index: *abcdef,
                        waiting: Default::default(),
                    })
                ]),
            },
        );
        assert_eq!(res.cache.entries.len(), 6);
    }

}
