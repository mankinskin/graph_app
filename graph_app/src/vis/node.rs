use context_trace::graph::{
    vertex::{
        child::Child,
        data::VertexData,
        has_vertex_index::HasVertexIndex,
        key::VertexKey,
        pattern::id::PatternId,
        wide::Wide,
    },
    Hypergraph,
};
use eframe::{
    egui::{
        self,
        Color32,
        Frame,
        Response,
        Style,
        Ui,
        Vec2,
        Window,
    },
    epaint::Shadow,
};
use petgraph::graph::NodeIndex;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};

use super::graph::GraphVis;
use crate::graph::Graph;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct NodeVis {
    key: VertexKey,
    idx: NodeIndex,
    name: String,
    data: VertexData,
    child_patterns: ChildPatternsVis,
    state: Arc<RwLock<NodeState>>,
    graph: Graph,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct NodeState {
    split_lower: usize,
    split_upper: usize,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            split_lower: 1,
            split_upper: 7,
        }
    }
}

impl std::ops::Deref for NodeVis {
    type Target = VertexData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl NodeVis {
    pub fn new(
        graph: Graph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey,
        data: &VertexData,
    ) -> Self {
        Self::new_impl(
            graph,
            node_indices,
            idx,
            key,
            data,
            Arc::new(RwLock::new(NodeState::new())),
        )
    }
    pub fn from_old(
        old: &NodeVis,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        idx: NodeIndex,
        data: &VertexData,
    ) -> Self {
        Self::new_impl(
            old.graph.clone(),
            node_indices,
            idx,
            &old.key,
            data,
            old.state.clone(),
        )
    }
    pub fn new_impl(
        graph: Graph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey,
        data: &VertexData,
        state: Arc<RwLock<NodeState>>,
    ) -> Self {
        let (name, child_patterns) = {
            let graph = &*graph.read();
            let name = graph.vertex_data_string(data);
            let child_patterns =
                Self::child_patterns_vis(graph, node_indices, data);
            (name, child_patterns)
        };
        Self {
            key: *key,
            graph,
            idx,
            name,
            data: data.clone(),
            child_patterns,
            state,
        }
    }
    #[allow(unused)]
    fn state(&self) -> RwLockReadGuard<'_, NodeState> {
        self.state.read().unwrap()
    }
    #[allow(unused)]
    fn state_mut(&self) -> RwLockWriteGuard<'_, NodeState> {
        self.state.write().unwrap()
    }
    fn child_patterns_vis(
        graph: &Hypergraph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        data: &VertexData,
    ) -> ChildPatternsVis {
        data.get_child_patterns()
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
            .collect()
    }
    pub fn child_patterns(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> Response {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.child_patterns.iter().for_each(|(_pid, cpat)| {
                let r = self.measure_pattern(ui, cpat, gvis);
                let height = r.rect.height();
                self.pattern(ui, cpat, Some(height), gvis);
            })
        })
        .response
    }
    fn measure_pattern(
        &self,
        ui: &mut Ui,
        pat: &PatternVis,
        gvis: &GraphVis,
    ) -> Response {
        let old_clip_rect = ui.clip_rect();
        let old_cursor = ui.cursor();
        ui.set_clip_rect(egui::Rect::NOTHING);
        let r = self.pattern(ui, pat, None, gvis);
        ui.set_clip_rect(old_clip_rect);
        ui.set_cursor(old_cursor);
        r
    }
    fn pattern(
        &self,
        ui: &mut Ui,
        pat: &PatternVis,
        height: Option<f32>,
        gvis: &GraphVis,
    ) -> Response {
        ui.horizontal(|ui| {
            if let Some(height) = height {
                ui.set_min_height(height);
            }
            pat.pattern.iter().for_each(|child| {
                self.child_index(ui, child, gvis);
            })
        })
        .response
    }
    fn child_index(
        &self,
        ui: &mut Ui,
        child: &ChildVis,
        gvis: &GraphVis,
    ) -> Response {
        Frame::group(&Style::default())
            .inner_margin(3.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                //ui.set_min_width(UNIT_WIDTH * child.width as f32);
                if gvis.layout.is_nested() && child.idx.is_some() {
                    assert!(child.child.width() > 1);
                    let node = gvis
                        .graph
                        .node_weight(child.idx.unwrap())
                        .expect("Invalid NodeIndex in ChildVis!");
                    node.child_patterns(ui, gvis)
                } else {
                    ui.monospace(&child.name)
                }
            })
            .response
    }
    pub fn show(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> Option<Response> {
        Window::new(format!("{}({})", self.name, self.idx.index()))
            //Window::new(&self.name)
            .vscroll(true)
            .auto_sized()
            //.default_width(80.0)
            .frame(
                Frame::window(&Style::default()).shadow(Shadow::NONE).fill(
                    self.graph
                        .labels
                        .read()
                        .unwrap()
                        .contains(&self.key)
                        .then_some(Color32::from_rgb(10, 50, 10))
                        .unwrap_or(ui.style().visuals.widgets.open.bg_fill),
                ),
            )
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                self.child_patterns(ui, gvis)
            })
            .map(|ir| ir.response)
    }
}

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
}

#[derive(Clone, Debug)]
struct PatternVis {
    pattern: Vec<ChildVis>,
}

impl PatternVis {
    fn new(pattern: Vec<ChildVis>) -> Self {
        Self { pattern }
    }
}

type ChildPatternsVis = Vec<(PatternId, PatternVis)>;
