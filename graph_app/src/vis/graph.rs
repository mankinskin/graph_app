use context_trace::{
    graph::vertex::{
        has_vertex_key::HasVertexKey,
        location::pattern::PatternLocation,
        wide::Wide,
    },
    End,
    HashMap,
    IndexRangePath,
    IndexRoot,
    RolePath,
    Start,
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
use crate::{
    graph::Graph,
    vis::{
        layout::GraphLayout,
        node::SelectionState,
    },
};

#[derive(Default, Debug)]
pub struct GraphVis {
    graph: DiGraph<NodeVis, ()>,
    handle: Option<Graph>,
    layout: GraphLayout,
}
#[derive(Debug)]
#[allow(unused)]
pub enum UpdateError {
    NoRecordingStream,
    Stream(rerun::RecordingStreamError),
    NotInitialized,
}
use UpdateError::*;
impl GraphVis {
    fn update_rerun(
        &mut self,
        handle: &Graph,
    ) -> Result<(), UpdateError> {
        let rec = handle.rec.as_ref().ok_or(NoRecordingStream)?;
        rec.log_static("/graph", &self.layout.re_nodes())
            .map_err(Stream)?;

        rec.log_static("/graph", &self.layout.re_edges())
            .map_err(Stream)?;
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), UpdateError> {
        let handle = self.graph().ok_or(NotInitialized)?;
        let cg = handle.read();
        let pg = cg.to_petgraph().filter_map_owned(
            |_idx, (_index, node)| {
                if node.data.width() <= 1 {
                    None
                } else {
                    Some((node.data.vertex_key(), node))
                }
            },
            |_idx, e| (e.token.width() > 1).then_some(()),
        );
        if !self.initialized() {
            self.layout = GraphLayout::generate(&cg, pg);
        }
        self.update_rerun(&handle)?;
        self.update_egui(&handle);
        Ok(())
    }
    fn update_egui(
        &mut self,
        handle: &Graph,
    ) {
        if !self.initialized() {
            self.graph = self.layout.graph.clone().map_owned(
                |i, (k, n)| {
                    let vis = NodeVis::new(
                        handle.clone(),
                        i,
                        &k,
                        &n.data,
                        *self
                            .layout
                            .nodes
                            .iter_mut()
                            .zip(self.layout.positions.iter())
                            .find(|((_key, (id, _n)), _pos)| *id == i)
                            .unwrap()
                            .1,
                    );
                    vis
                },
                |_, e| e,
            );
        }
        for ((key, (i, node)), pos) in self
            .layout
            .nodes
            .iter_mut()
            .zip(self.layout.positions.iter())
        {
            if let Some(old) = self.graph.node_weight_mut(*i) {
                *old = NodeVis::from_old(old, *i, &node.data);
            } else {
                let vis =
                    NodeVis::new(handle.clone(), *i, key, &node.data, *pos);
                *i = self.graph.add_node(vis);
            };
        }
    }
    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) {
        if !self.initialized() {
            if let Err(err) = self.update() {
                println!("Error updating graph: {:?}", err);
            }
        }
        let _events = self.poll_events();

        // Get the available rect for constraining node windows
        let viewport_rect = ui.available_rect_before_wrap();

        //println!("{}", self.graph.vertex_count());
        let node_responses: HashMap<_, _> = self
            .graph
            .nodes()
            .filter_map(|(idx, node)| {
                node.show(ui, viewport_rect).map(|response| (idx, response))
            })
            .collect();

        // Use a clipped painter for edges so they don't render over the panels
        let clipped_painter = ui.painter().with_clip_rect(viewport_rect);

        self.graph.edge_references().for_each(|edge| {
            if let Some((ra, rb)) = node_responses
                .get(&edge.source())
                .zip(node_responses.get(&edge.target()))
            {
                let a_pos = ra.response.rect.center();
                let b = rb.response.rect;
                let p = Self::border_intersection_point(&b, &a_pos);
                Self::edge_clipped(&clipped_painter, &a_pos, &p);
            }
        });
        for (idx, response) in node_responses.into_iter() {
            let node = self
                .graph
                .node_weight_mut(idx)
                .expect("Invalid NodeIndex in node_responses!");
            node.selected_range = response
                .ranges
                .into_iter()
                .find_map(|(pid, r)| {
                    if let Some(state) = node.selected_range.as_mut() {
                        (state.pattern_id == pid).then_some((pid, r))
                    } else {
                        Some((pid, r))
                    }
                })
                .map(|(pid, range)| SelectionState {
                    pattern_id: pid,
                    trace: IndexRangePath::new(
                        IndexRoot::from(PatternLocation {
                            parent: node.data.to_child(),
                            pattern_id: pid,
                        }),
                        RolePath::<Start>::new_empty(range.start),
                        RolePath::<End>::new_empty(range.end),
                    ),
                    range,
                });
            if let Some(state) = node.selected_range.as_ref() {
                Window::new("Range").show(ui.ctx(), |ui| {
                    ui.label(format!("{:#?}", state));
                });
            }
        }
    }
    fn initialized(&self) -> bool {
        self.graph.node_count() > 0 && self.handle.is_some()
    }
    pub fn new(graph: Graph) -> Self {
        Self {
            graph: DiGraph::new(),
            handle: Some(graph),
            layout: GraphLayout::default(),
        }
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
    fn edge_tip_clipped(
        painter: &egui::Painter,
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
        painter.add(Shape::convex_polygon(
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
    fn edge_clipped(
        painter: &egui::Painter,
        source: &Pos2,
        target: &Pos2,
    ) {
        painter.add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
        Self::edge_tip_clipped(painter, source, target, 10.0);
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
