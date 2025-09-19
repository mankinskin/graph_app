use context_trace::{
    graph::vertex::{
        data::VertexData,
        key::VertexKey,
    },
    HashMap,
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
use std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

use super::graph::GraphVis;
use crate::{
    graph::Graph,
    vis::pattern::{
        ChildPatternsResponse,
        ChildPatternsVis,
    },
};

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

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct NodeVis {
    key: VertexKey,
    idx: NodeIndex,
    name: String,
    data: VertexData,
    pub(crate) child_patterns: ChildPatternsVis,
    state: Arc<RwLock<NodeState>>,
    graph: Graph,
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
                ChildPatternsVis::new(graph, node_indices, data);
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
                let ChildPatternsResponse { response, ranges } =
                    self.child_patterns.show(ui, gvis);
                if ranges.iter().any(|r| r.is_some()) {
                    Window::new("Ranges").show(ui.ctx(), |ui| {
                        for r in &ranges {
                            ui.label(format!("{:?}", r));
                        }
                    });
                }
                response
            })
            .map(|ir| ir.response)
    }
}
