use std::ops::Range;

use context_trace::{
    graph::{
        vertex::{
            child::Child,
            data::VertexData,
            has_vertex_index::HasVertexIndex,
            key::VertexKey,
            pattern::id::PatternId,
            wide::Wide,
        },
        Hypergraph,
    },
    HashMap,
};
use eframe::egui::{
    self,
    Frame,
    Response,
    Style,
    Ui,
    Vec2,
};
use petgraph::graph::NodeIndex;

use crate::vis::graph::GraphVis;

#[derive(Clone, Debug)]
struct ChildVis {
    name: String,
    child: Child,
    idx: Option<NodeIndex>,
}

impl std::ops::Deref for ChildVis {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
pub struct ChildResponse {
    #[allow(unused)]
    pub response: Response,
    pub range: Option<Range<usize>>,
}

impl ChildVis {
    fn new(
        graph: &Hypergraph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        child: Child,
    ) -> Self {
        let key = graph.expect_key_for_index(child.index);
        let name = graph.index_string(child.vertex_index());
        let idx = node_indices.get(&key).cloned();
        Self { name, child, idx }
    }
    fn show(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> ChildResponse {
        let response =
            Frame::group(&Style::default())
                .inner_margin(3.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                    //ui.set_min_width(UNIT_WIDTH * child.width as f32);
                    if gvis.layout.is_nested() && self.idx.is_some() {
                        assert!(self.child.width() > 1);
                        let node = gvis
                            .graph
                            .node_weight(self.idx.unwrap())
                            .expect("Invalid NodeIndex in ChildVis!");
                        node.child_patterns
                            .show(ui, gvis)
                            .ranges
                            .into_iter()
                            .find_map(|r| r)
                    } else {
                        ui.monospace(&self.name).selected_char_range()
                    }
                });
        ChildResponse {
            response: response.response,
            range: response.inner,
        }
    }
}

#[derive(Clone, Debug)]
struct PatternVis {
    pattern: Vec<ChildVis>,
}
pub struct PatternResponse {
    pub response: Response,
    pub ranges: Vec<Option<Range<usize>>>,
}
impl PatternVis {
    fn new(pattern: Vec<ChildVis>) -> Self {
        Self { pattern }
    }
    fn measure(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> PatternResponse {
        let old_clip_rect = ui.clip_rect();
        let old_cursor = ui.cursor();
        ui.set_clip_rect(egui::Rect::NOTHING);
        let r = self.show(ui, None, gvis);
        ui.set_clip_rect(old_clip_rect);
        ui.set_cursor(old_cursor);
        r
    }
    fn show(
        &self,
        ui: &mut Ui,
        height: Option<f32>,
        gvis: &GraphVis,
    ) -> PatternResponse {
        let response = ui.horizontal(|ui| {
            if let Some(height) = height {
                ui.set_min_height(height);
            }
            self.pattern
                .iter()
                .map(|child| child.show(ui, gvis).range)
                .collect()
        });
        PatternResponse {
            response: response.response,
            ranges: response.inner,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChildPatternsVis {
    patterns: Vec<(PatternId, PatternVis)>,
}
#[derive(Clone, Debug)]
pub struct ChildPatternsResponse {
    pub response: Response,
    pub ranges: Vec<Option<Range<usize>>>,
}

impl ChildPatternsVis {
    pub fn new(
        graph: &Hypergraph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        data: &VertexData,
    ) -> Self {
        Self {
            patterns: data
                .get_child_patterns()
                .iter()
                .map(|(&id, pat)| {
                    (
                        id,
                        PatternVis::new(
                            pat.iter()
                                .map(|c| ChildVis::new(graph, node_indices, *c))
                                .collect(),
                        ),
                    )
                })
                .collect(),
        }
    }
    pub fn show(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> ChildPatternsResponse {
        let response = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.patterns
                .iter()
                .map(|(_pid, cpat)| {
                    let r = cpat.measure(ui, gvis);
                    let height = r.response.rect.height();
                    let ranges = cpat.show(ui, Some(height), gvis).ranges;

                    cpat.pattern
                        .iter()
                        .scan(0, |acc, c| {
                            let prev = *acc;
                            *acc += c.child.width();
                            Some(prev)
                        })
                        .zip(ranges)
                        .fold(None as Option<Range<usize>>, |acc, (off, r)| {
                            let o = r.map(|r| (r.start + off)..(r.end + off));
                            acc.as_ref()
                                .zip(o.as_ref())
                                .map(|(acc, o)| {
                                    acc.start.min(o.start)..acc.end.max(o.end)
                                })
                                .or(acc)
                                .or(o)
                        })
                })
                .collect::<Vec<_>>()
        });
        ChildPatternsResponse {
            response: response.response,
            ranges: response.inner,
        }
    }
}
