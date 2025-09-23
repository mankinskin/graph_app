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
    Window,
};
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
    datatypes::Utf8,
    GraphEdges,
    GraphNodes,
    Vec2D,
};
use std::f32::consts::PI;

use super::node::NodeVis;
use crate::graph::Graph;

#[derive(Default, Debug)]
pub struct GraphVis {
    graph: DiGraph<NodeVis, ()>,
    handle: Option<Graph>,
    pub(crate) graph_pos: Pos2,
}
pub enum UpdateError {
    NoRecordingStream,
    Stream(rerun::RecordingStreamError),
    NotInitialized,
}
use UpdateError::*;
impl GraphVis {
    pub fn update(&mut self) -> Result<(), UpdateError> {
        // todo reuse names in nodes
        let handle = self.graph().ok_or(NotInitialized)?;
        let cg = handle.read();
        let pg = cg.to_petgraph();
        let rec = handle.rec.as_ref().ok_or(NoRecordingStream)?;
        //let coordinates = (0..NUM_NODES).cartesian_product(0..NUM_NODES);

        //let (nodes, colors): (Vec<_>, Vec<_>) = coordinates
        //    .clone()
        //    .enumerate()
        //    .map(|(i, (x, y))| {
        //        use rerun::Color;

        //        let r =
        //            ((x as f32 / (NUM_NODES - 1) as f32) * 255.0).round() as u8;
        //        let g =
        //            ((y as f32 / (NUM_NODES - 1) as f32) * 255.0).round() as u8;
        //        (i.to_string(), Color::from_rgb(r, g, 0))
        //    })
        //    .unzip();

        let pg = pg.filter_map(
            |_idx, (_index, node)| {
                if node.data.width() <= 1 {
                    None
                } else {
                    Some((node.data.key, node))
                }
            },
            |_idx, e| (e.child.width() > 1).then_some(()),
        );
        let nodes = pg.node_weights();
        let indices = pg.node_indices().map(|i| format!("{:?}", i));
        let labels = nodes
            .map(|(i, n)| {
                let name = n.name.clone();
                let patterns = n.data.to_pattern_strings(&cg);
                format!(
                    "{}\n{}",
                    name,
                    patterns.into_iter().map(|v| v.join(" ")).join("\n")
                )
            })
            .collect_vec();

        let n = pg.node_count();
        let c = (n as f32).sqrt().ceil();
        let s = 180.0;
        let h = 120.0;
        let w = c * s;
        let x = 0.0;
        let y = 0.0;
        let positions = (0..n).map(|i| {
            Vec2D::new(
                x + (i as f32 * s) % w,
                y + ((i as f32 * s) / w).floor() * h,
            )
        });

        rec.log_static(
            "/graph",
            &GraphNodes::new(indices)
                .with_labels(labels)
                .with_positions(positions),
        )
        .map_err(Stream)?;

        //let mut edges = Vec::new();
        //for (x, y) in coordinates {
        //    if y > 0 {
        //        let source = (y - 1) * NUM_NODES + x;
        //        let target = y * NUM_NODES + x;
        //        edges.push((source.to_string(), target.to_string()));
        //    }
        //    if x > 0 {
        //        let source = y * NUM_NODES + (x - 1);
        //        let target = y * NUM_NODES + x;
        //        edges.push((source.to_string(), target.to_string()));
        //    }
        //}

        let edges = pg.edge_references().map(|e| {
            (format!("{:?}", e.source()), format!("{:?}", e.target()))
        });
        rec.log_static("/graph", &GraphEdges::new(edges).with_directed_edges())
            .map_err(Stream)?;
        //let old_node_indices: HashMap<_, _> = self
        //    .graph
        //    .nodes()
        //    .map(|(idx, node)| (node.key, idx))
        //    .collect();
        //let filtered = pg.filter_map(
        //    |_idx, (_index, node)| {
        //        if node.width() <= 1 {
        //            None
        //        } else {
        //            Some((node.key, node))
        //        }
        //    },
        //    |_idx, e| (e.child.width() > 1).then_some(()),
        //);
        //self.graph = filtered.map(
        //    |idx, (key, node)| {
        //        if let Some(oid) = old_node_indices.get(key) {
        //            let old = self.graph.node_weight(*oid).unwrap();
        //            NodeVis::from_old(old, idx, node)
        //        } else {
        //            NodeVis::new(
        //                handle.clone(),
        //                idx,
        //                key,
        //                node,
        //                pos_generator.next().unwrap(),
        //            )
        //        }
        //    },
        //    |_idx, _e| (),
        //);
        Ok(())
    }
    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) {
        if !self.initialized() {
            self.update();
        }
        let _events = self.poll_events();
        //println!("{}", self.graph.vertex_count());
        let node_responses: HashMap<_, _> = self
            .graph
            .nodes()
            .filter_map(|(idx, node)| {
                node.show(ui).map(|response| (idx, response))
            })
            .collect();
        self.graph.edge_references().for_each(|edge| {
            node_responses
                .get(&edge.source())
                .zip(node_responses.get(&edge.target()))
                .map(|(ra, rb)| {
                    let a_pos = ra.response.rect.center();
                    let b = rb.response.rect;
                    let p = Self::border_intersection_point(&b, &a_pos);
                    Self::edge(ui, &a_pos, &p);
                });
        });
        for (idx, response) in node_responses.into_iter() {
            let node = self
                .graph
                .node_weight_mut(idx)
                .expect("Invalid NodeIndex in node_responses!");
            node.selected_range =
                response.ranges.into_iter().find_map(|(pid, r)| {
                    if let Some((spid, _)) = node.selected_range.as_mut() {
                        (*spid == pid).then(|| (pid, r))
                    } else {
                        Some((pid, r))
                    }
                });
            if let Some((pid, r)) = node.selected_range.as_ref() {
                Window::new("Range").show(ui.ctx(), |ui| {
                    ui.label(format!("{}: {:?}", pid, r));
                });
            }
        }
    }
    fn initialized(&self) -> bool {
        self.graph.node_count() > 0 && self.handle.is_some()
    }
    pub fn new(graph: Graph) -> Self {
        let mut new = Self {
            graph: DiGraph::new(),
            handle: Some(graph),
            graph_pos: Pos2::default(),
        };
        new.update();
        new
    }
    pub fn set_graph(
        &mut self,
        graph: Graph,
    ) {
        self.handle = Some(graph);
        self.update();
    }
    fn graph(&self) -> Option<Graph> {
        self.handle.clone()
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
