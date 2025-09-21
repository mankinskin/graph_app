use std::ops::Range;

use context_trace::graph::vertex::{
    data::VertexData,
    key::VertexKey,
    pattern::id::PatternId,
};
use eframe::{
    egui::{
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

use crate::{
    graph::Graph,
    vis::pattern::ChildPatternsVis,
};

pub struct NodeResponse {
    pub response: Response,
    pub ranges: Vec<(PatternId, Range<usize>)>,
}
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct NodeVis {
    key: VertexKey,
    idx: NodeIndex,
    name: String,
    data: VertexData,
    pub(crate) child_patterns: ChildPatternsVis,
    graph: Graph,
    pub selected_range: Option<(PatternId, Range<usize>)>,
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
        idx: NodeIndex,
        key: &VertexKey,
        data: &VertexData,
    ) -> Self {
        Self::new_impl(graph, idx, key, data, None)
    }
    pub fn from_old(
        old: &NodeVis,
        idx: NodeIndex,
        data: &VertexData,
        selected_range: Option<(PatternId, Range<usize>)>,
    ) -> Self {
        Self::new_impl(old.graph.clone(), idx, &old.key, data, selected_range)
    }
    pub fn new_impl(
        graph: Graph,
        idx: NodeIndex,
        key: &VertexKey,
        data: &VertexData,
        selected_range: Option<(PatternId, Range<usize>)>,
    ) -> Self {
        let (name, child_patterns) = {
            let graph = &*graph.read();
            let name = graph.vertex_data_string(data);
            let child_patterns = ChildPatternsVis::new(graph, data);
            (name, child_patterns)
        };
        Self {
            key: *key,
            graph,
            idx,
            name,
            data: data.clone(),
            child_patterns,
            selected_range,
        }
    }
    pub fn show(
        &self,
        ui: &mut Ui,
    ) -> Option<NodeResponse> {
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
                self.child_patterns.show(ui).ranges
            })
            .and_then(|ir| {
                ir.inner.map(|inner| NodeResponse {
                    response: ir.response,
                    ranges: inner,
                })
            })
    }
}
