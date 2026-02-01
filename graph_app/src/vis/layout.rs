use context_trace::{
    graph::{
        vertex::{key::VertexKey, wide::Wide},
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
    /// Positions indexed by NodeIndex
    pub(crate) positions: HashMap<NodeIndex, Pos2>,
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
        
        // Sort nodes by width (size) in descending order - largest first
        let mut sorted_nodes: Vec<_> = nodes.iter().collect();
        sorted_nodes.sort_by(|a, b| {
            let width_a = a.1.1.data.width();
            let width_b = b.1.1.data.width();
            width_b.cmp(&width_a) // Descending order
        });
        
        let indices = sorted_nodes
            .iter()
            .map(|(_k, (i, _n))| format!("{:?}", i))
            .collect_vec();
        let labels = sorted_nodes
            .iter()
            .map(|(_k, (_i, n))| {
                let name = n.name.clone();
                let patterns = n.data.to_pattern_strings(cg);
                format!(
                    "{}\n{}",
                    name,
                    patterns.into_iter().map(|v| v.join(" ")).join("\n")
                )
            })
            .collect_vec();

        // Calculate node sizes dynamically based on label content
        // Estimate character width and line height for sizing
        let char_width = 8.0; // Approximate pixels per character
        let line_height = 18.0; // Approximate pixels per line
        let padding = 40.0; // Padding inside node window
        let min_node_width = 150.0;
        let min_node_height = 60.0;
        
        // Calculate sizes for each node based on label content
        let mut node_sizes: HashMap<NodeIndex, (f32, f32)> = HashMap::default();
        for ((_key, (node_idx, node)), label) in sorted_nodes.iter().zip(labels.iter()) {
            let lines: Vec<&str> = label.lines().collect();
            let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(10);
            let num_lines = lines.len();
            
            let width = (max_line_len as f32 * char_width + padding).max(min_node_width);
            let height = (num_lines as f32 * line_height + padding).max(min_node_height);
            node_sizes.insert(*node_idx, (width, height));
        }
        
        // Layout settings
        let h_spacing = 50.0; // Horizontal spacing between nodes
        let v_spacing = 50.0; // Vertical spacing between rows
        let max_row_width = 2500.0; // Maximum width before wrapping
        
        // Starting offset to account for UI panels/margins
        let start_x = 60.0;
        let start_y = 60.0;
        
        // Group nodes by their token width
        let mut width_groups: std::collections::BTreeMap<_, Vec<_>> = std::collections::BTreeMap::new();
        for (key, (node_idx, node)) in sorted_nodes.iter() {
            let w = node.data.width();
            width_groups.entry(w).or_default().push((*key, *node_idx));
        }
        
        let mut positions = HashMap::default();
        let mut current_y = start_y;
        
        // Iterate from largest width to smallest (reverse order of BTreeMap)
        for (_width, group) in width_groups.into_iter().rev() {
            let mut current_x = start_x;
            let mut max_row_height = 0.0f32;
            
            for (_key, node_idx) in group.iter() {
                let (node_w, node_h) = node_sizes.get(node_idx).copied().unwrap_or((min_node_width, min_node_height));
                
                // Check if we need to wrap to next row within this width group
                if current_x + node_w > max_row_width && current_x > start_x {
                    current_x = start_x;
                    current_y += max_row_height + v_spacing;
                    max_row_height = 0.0;
                }
                
                positions.insert(*node_idx, Pos2::new(current_x, current_y));
                current_x += node_w + h_spacing;
                max_row_height = max_row_height.max(node_h);
            }
            
            // Move to next row for the next width group
            current_y += max_row_height + v_spacing;
        }
        
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
        // Get positions in the same order as indices
        let positions_vec: Vec<_> = self.indices.iter()
            .filter_map(|idx_str| {
                // Parse the index string back to NodeIndex
                let idx_num: usize = idx_str.trim_start_matches("NodeIndex(")
                    .trim_end_matches(")")
                    .parse().ok()?;
                self.positions.get(&NodeIndex::new(idx_num))
                    .map(|p| Vec2D::new(p.x, p.y))
            })
            .collect();
        
        GraphNodes::new(self.indices.clone())
            .with_labels(self.labels.clone())
            .with_positions(positions_vec)
    }
    pub fn re_edges(&self) -> GraphEdges {
        GraphEdges::new(self.edges.clone()).with_directed_edges()
    }
}
