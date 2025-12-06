use context_trace::{
    graph::{
        vertex::key::VertexKey,
        Hypergraph,
    },
    HashMap,
};

use eframe::egui::Pos2;
use itertools::Itertools;
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};
use rerun::{
    GraphEdges,
    GraphNodes,
    Vec2D,
};

type GraphData = petgraph::Graph<
    (
        context_trace::graph::vertex::key::VertexKey,
        context_trace::graph::Node,
    ),
    (),
>;
#[derive(Default, Debug)]
pub struct GraphLayout {
    pub(crate) labels: Vec<String>,
    pub(crate) indices: Vec<String>,
    pub(crate) nodes:
        HashMap<VertexKey, (NodeIndex, context_trace::graph::Node)>,
    pub(crate) positions: Vec<Pos2>,
    pub(crate) edges: Vec<(String, String)>,
    pub(crate) graph: GraphData,
}

impl GraphLayout {
    pub fn generate(
        cg: &Hypergraph,
        pg: GraphData,
    ) -> Self {
        // todo reuse names in nodes
        let (nodes, edges) = pg
            .clone()
            .map_owned(|i, (k, n)| (k, (i, n)), |_, e| e)
            .into_nodes_edges();
        let nodes: HashMap<_, _> =
            nodes.into_iter().map(|node| node.weight).collect();
        let indices = nodes
            .iter()
            .map(|(_k, (i, _n))| format!("{:?}", i))
            .collect_vec();
        let labels = nodes
            .iter()
            .map(|(_k, (_i, n))| {
                let name = n.name.clone();
                let patterns = n.data.to_pattern_strings(&cg);
                format!(
                    "{}\n{}",
                    name,
                    patterns.into_iter().map(|v| v.join(" ")).join("\n")
                )
            })
            .collect_vec();

        let n = nodes.len();
        let c = (n as f32).sqrt().ceil();
        let s = 180.0;
        let h = 120.0;
        let w = c * s;
        let x = 0.0;
        let y = 0.0;
        let positions = (0..n)
            .map(|i| {
                Pos2::new(
                    x + (i as f32 * s) % w,
                    y + ((i as f32 * s) / w).floor() * h,
                )
            })
            .collect_vec();
        let edges = edges
            .into_iter()
            .map(|e| (format!("{:?}", e.source()), format!("{:?}", e.target())))
            .collect_vec();

        Self {
            graph: pg,
            nodes,
            edges,
            indices,
            labels,
            positions,
        }
    }
    pub fn re_nodes(&self) -> GraphNodes {
        GraphNodes::new(self.indices.clone())
            .with_labels(self.labels.clone())
            .with_positions(
                self.positions
                    .clone()
                    .into_iter()
                    .map(|p| Vec2D::new(p.x, p.y)),
            )
    }
    pub fn re_edges(&self) -> GraphEdges {
        GraphEdges::new(self.edges.clone()).with_directed_edges()
    }
}
