use crate::*;

pub type DirectedPositions = HashMap<TokenLocation, PositionCache>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VertexCache {
    pub(crate) bottom_up: DirectedPositions,
    pub(crate) top_down: DirectedPositions,
    pub(crate) index: Child,
}
impl From<Child> for VertexCache {
    fn from(index: Child) -> Self {
        Self {
            index,
            bottom_up: Default::default(),
            top_down: Default::default(),
        }
    }
}
impl VertexCache {
    pub fn start(index: Child) -> Self {
        let bottom_up = HashMap::default();
        //bottom_up.insert(
        //    index.width().into(),
        //    PositionCache::start(index)
        //);
        Self {
            bottom_up,
            index,
            top_down: Default::default(),
        }
    }
    pub fn dir(&self, pos: &DirectedPosition) -> &DirectedPositions {
        match pos {
            DirectedPosition::BottomUp(_) => &self.bottom_up,
            DirectedPosition::TopDown(_) => &self.top_down,
        }
    }
    pub fn dir_mut(&mut self, pos: &DirectedPosition) -> &mut DirectedPositions {
        match pos {
            DirectedPosition::BottomUp(_) => &mut self.bottom_up,
            DirectedPosition::TopDown(_) => &mut self.top_down,
        }
    }
    pub fn get(&self, pos: &DirectedPosition) -> Option<&PositionCache> {
        self.dir(pos).get(pos.pos())
    }
    pub fn get_mut(&mut self, pos: &DirectedPosition) -> Option<&mut PositionCache> {
        self.dir_mut(pos).get_mut(pos.pos())
    }
    pub fn insert(&mut self, pos: &DirectedPosition, cache: PositionCache) {
        self.dir_mut(pos).insert(
            *pos.pos(),
            cache,
        );
    }
    pub fn global_splits<N: NodeType>(
        &self,
        end_pos: TokenLocation,
        node: &VertexData,
    ) -> N::GlobalSplitOutput {
        let mut output = N::GlobalSplitOutput::default();
        let (mut front, mut back) = (false, false);
        for (inner_width, cache) in &self.bottom_up {
            for location in cache.edges.bottom.values() {
                let child = node.expect_child_at(location);
                let inner_offset = Offset::new(child.width() - inner_width.0);
                let bottom = SubSplitLocation {
                    location: *location,
                    inner_offset,
                };
                let offset = node.expect_child_offset(location);
                if let Some(parent_offset) = inner_offset
                    .and_then(|o| o.checked_add(offset))
                    .or(NonZeroUsize::new(offset))
                {
                    output.splits_mut()
                        .entry(parent_offset)
                        .and_modify(|e: &mut Vec<_>|
                            e.push(bottom.clone())
                        )
                        .or_insert_with(||
                            vec![bottom]
                        );
                    front = true;
                } else {
                    break;
                }
            }
        }
        for (pretext_pos, cache) in &self.top_down {
            let inner_offset = Offset::new(end_pos.0 - pretext_pos.0).unwrap();
            for location in cache.edges.bottom.values() {
                let child = node.expect_child_at(location);
                let inner_offset = Offset::new(inner_offset.get() % child.width());
                let location = SubLocation {
                    sub_index: location.sub_index + inner_offset.is_none() as usize,
                    pattern_id: location.pattern_id,
                };
                let bottom = SubSplitLocation {
                    location,
                    inner_offset,
                };
                let offset = node.expect_child_offset(&location);
                let parent_offset = inner_offset.map(|o| o.checked_add(offset).unwrap())
                    .unwrap_or_else(|| NonZeroUsize::new(offset).unwrap());
                if parent_offset.get() < node.width {
                    if let Some(e) = output.splits_mut().get_mut(&parent_offset) {
                        e.push(bottom)
                    } else {
                        output.splits_mut().insert(
                            parent_offset,
                            vec![bottom]
                        );
                    }
                    back = true;
                }
            }
        }
        match (front, back) {
            (true, true) => output.set_root_mode(RootMode::Infix),
            (false, true) => output.set_root_mode(RootMode::Prefix),
            (true, false) => output.set_root_mode(RootMode::Postfix),
            (false, false) => unreachable!(),
        }
        output
    }
    pub fn complete_splits<Trav: Traversable, N: NodeType>(
        &self,
        trav: &Trav,
        end_pos: TokenLocation,
    ) -> N::CompleteSplitOutput {
        let graph = trav.graph();

        let (_, node) = graph.expect_vertex(self.index);

        let output = self.global_splits::<N>(end_pos, node);

        N::map(output, |global_splits|
            global_splits.into_iter()
                .map(|(parent_offset, mut locs)| {
                    if locs.len() < node.children.len() {
                        let pids: HashSet<_> = locs.iter().map(|l| l.location.pattern_id).collect();
                        let missing = node.children.iter()
                            .filter(|(pid, _)|
                                !pids.contains(pid)
                            );
                        locs.extend(
                            position_splits(
                                missing,
                                parent_offset,
                            )
                            .splits
                            .into_iter()
                            .map(|(pid, loc)|
                                SubSplitLocation {
                                    location: SubLocation::new(
                                        pid,
                                        loc.sub_index,
                                    ),
                                    inner_offset: loc.inner_offset,
                                }
                            )
                        )
                    }
                    (
                        parent_offset,
                        locs.into_iter().map(|sub|
                            if sub.inner_offset.is_some() || node.children[&sub.location.pattern_id].len() > 2 {
                                // can't be clean
                                Ok(sub)
                            } else {
                                // must be clean
                                Err(sub.location)
                            }
                        ).collect()
                    )
            }).collect()
        )
    }
}