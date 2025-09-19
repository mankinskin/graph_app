use context_trace::{
    graph::vertex::wide::Wide,
    HashMap,
};
use eframe::egui::{
    self,
    vec2,
    Pos2,
    Rect,
    Shape,
    Stroke,
    Ui,
    Vec2,
};
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};
use std::f32::consts::PI;

use super::node::NodeVis;
use crate::graph::Graph;

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Graph,
    Nested,
}
impl Layout {
    #[allow(unused)]
    pub fn is_graph(&self) -> bool {
        matches!(self, Self::Graph)
    }
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::Nested)
    }
}
impl Default for Layout {
    fn default() -> Self {
        Self::Graph
    }
}
#[derive(Default, Debug)]
pub struct GraphVis {
    pub(crate) graph: DiGraph<NodeVis, ()>,
    pub layout: Layout,
    handle: Option<Graph>,
}
impl GraphVis {
    pub fn set_graph(
        &mut self,
        graph: Graph,
    ) {
        self.handle = Some(graph);
        self.update();
    }
    fn graph(&self) -> Graph {
        self.handle.clone().expect("GraphVis not yet initialized!")
    }
    pub fn update(&mut self) -> Option<()> {
        // todo reuse names in nodes
        //println!("update...");
        let pg = self.graph().read().to_petgraph();
        //println!("updating");
        let old_node_indices: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| (node.key, idx))
            .collect();
        let filtered = pg.filter_map(
            |_idx, (_index, node)| {
                if node.width() <= 1 {
                    None
                } else {
                    Some((node.key, node))
                }
            },
            |_idx, e| (e.child.width() > 1).then_some(()),
        );
        let node_indices: HashMap<_, _> = filtered
            .nodes()
            .map(|(idx, (key, _node))| (*key, idx))
            .collect();

        self.graph = filtered.map(
            |idx, (key, node)| {
                if let Some(oid) = old_node_indices.get(key) {
                    let old = self.graph.node_weight(*oid).unwrap();
                    NodeVis::from_old(old, &node_indices, idx, node)
                } else {
                    NodeVis::new(self.graph(), &node_indices, idx, key, node)
                }
            },
            |_idx, _e| (),
        );
        //println!("done");
        Some(())
    }
    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) {
        self.update();

        let _events = self.poll_events();
        //println!("{}", self.graph.vertex_count());
        let rects: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| {
                let response = node.show(ui, self).unwrap();
                (idx, response.rect)
            })
            .collect();
        self.graph.edge_references().for_each(|edge| {
            let a_pos = rects
                .get(&edge.source())
                .expect("No position for edge endpoint.")
                .center();
            let b = rects
                .get(&edge.target())
                .expect("No position for edge endpoint.");

            let p = Self::border_intersection_point(b, &a_pos);
            Self::edge(ui, &a_pos, &p);
        });
    }
    pub fn edge_tip(
        ui: &mut Ui,
        source: &Pos2,
        target: &Pos2,
        size: f32,
    ) {
        let angle = (*target - *source).angle();
        let points = IntoIterator::into_iter([
            Vec2::new(0.0, 0.0),
            Vec2::angled(angle - 0.25 * PI),
            Vec2::angled(angle + 0.25 * PI),
        ])
        .map(|p| *target - p * size)
        .collect();
        ui.painter().add(Shape::convex_polygon(
            points,
            egui::Color32::WHITE,
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
    }
    #[allow(unused)]
    pub fn edge(
        ui: &mut Ui,
        source: &Pos2,
        target: &Pos2,
    ) {
        ui.painter().add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
        Self::edge_tip(ui, source, target, 10.0);
    }
    #[allow(clippy::many_single_char_names)]
    fn border_intersection_point(
        rect: &Rect,
        p: &Pos2,
    ) -> Pos2 {
        let p = *p;
        let c = rect.center();
        let v = p - c;
        let s = v.y / v.x;
        let h = rect.height();
        let w = rect.width();
        c + if -h / 2.0 <= s * w / 2.0 && s * w / 2.0 <= h / 2.0 {
            // intersects side
            if p.x > c.x {
                // right
                vec2(w / 2.0, w / 2.0 * s)
            } else {
                // left
                vec2(-w / 2.0, -w / 2.0 * s)
            }
        } else {
            // intersects top or bottom
            if p.y > c.y {
                // top
                vec2(h / (2.0 * s), h / 2.0)
            } else {
                // bottom
                vec2(-h / (2.0 * s), -h / 2.0)
            }
        }
    }
    pub fn poll_events(&self) -> Vec<tracing_egui::LogEvent> {
        //println!("polling..");
        tracing_egui::poll_events().drain(..).collect()
    }
}
