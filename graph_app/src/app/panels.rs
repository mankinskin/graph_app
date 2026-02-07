//! Panel rendering (top, left, right, bottom).

use context_trace::{
    graph::vertex::{
        has_vertex_key::HasVertexKey,
        key::VertexKey,
    },
    VertexSet,
};
use eframe::egui::{
    self,
    Color32,
    PopupCloseBehavior,
    Pos2,
    Rect,
    Stroke,
    Vec2,
};
use egui::containers::menu::{
    MenuButton,
    MenuConfig,
};
use strum::IntoEnumIterator;

use super::App;
use crate::algorithm::Algorithm;

impl App {
    pub(crate) fn top_panel(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // View menu - use CloseOnClickOutside so checkboxes don't close the menu
                MenuButton::new("View")
                    .config(MenuConfig::new().close_behavior(
                        PopupCloseBehavior::CloseOnClickOutside,
                    ))
                    .ui(ui, |ui| {
                        ui.checkbox(&mut self.left_panel_open, "Left Panel");
                        ui.checkbox(&mut self.right_panel_open, "Right Panel");
                        ui.checkbox(
                            &mut self.bottom_panel_open,
                            "Bottom Panel",
                        );
                        ui.checkbox(&mut self.status_bar_open, "Status Bar");
                        ui.separator();
                        ui.checkbox(&mut self.inserter_open, "Inserter Window");
                        ui.checkbox(&mut self.settings_open, "Settings Window");
                        ui.separator();
                        if ui.button("New Tab").clicked() {
                            self.create_new_tab();
                            ui.close();
                        }
                    });

                // Right-aligned items
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("⚙").on_hover_text("Settings").clicked()
                        {
                            self.settings_open = !self.settings_open;
                        }
                    },
                );
            });
        });
    }

    pub(crate) fn left_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .min_width(150.0)
            .show_animated(ctx, self.left_panel_open, |ui| {
                ui.heading("Tools");
                ui.separator();

                // Algorithm selection
                ui.label("Algorithm:");
                egui::ComboBox::from_id_salt("left_panel_algorithm")
                    .selected_text(self.selected_algorithm.to_string())
                    .show_ui(ui, |ui| {
                        for algorithm in Algorithm::iter() {
                            ui.selectable_value(
                                &mut self.selected_algorithm,
                                algorithm,
                                algorithm.to_string(),
                            );
                        }
                    });

                ui.add_space(10.0);
                ui.label(self.selected_algorithm.description());

                ui.add_space(20.0);
                ui.separator();

                // Insert controls
                ui.heading("Insert");
                if let Some(mut read_ctx) = self.ctx_mut() {
                    for text in &mut read_ctx.graph_mut().insert_texts {
                        ui.text_edit_singleline(text);
                    }
                    if ui.button("+ Add Text").clicked() {
                        read_ctx.graph_mut().insert_texts.push(String::new());
                    }
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let is_running = self.is_task_running();
                    if ui.button("▶ Run").clicked() && !is_running {
                        self.start_read();
                    }
                    if is_running && ui.button("⏹ Cancel").clicked() {
                        self.abort();
                    }
                });
            });
    }

    pub(crate) fn right_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        // Get selected node from current tab
        let selected_node = self.current_tab().and_then(|t| t.selected_node);

        // Collect all the display data outside the panel closure
        let vertex_count = self
            .ctx()
            .and_then(|ctx| ctx.graph().try_read().map(|g| g.vertex_count()));

        // Collect selection info with more detail for rendering
        let selection_info = selected_node.and_then(|key| {
            let read_ctx = self.ctx()?;
            let graph = read_ctx.graph();
            let graph_ref = graph.try_read()?;
            let data = graph_ref.get_vertex_data(key).ok()?;

            let name = graph_ref.vertex_data_string(data.clone());
            let width = data.to_token().width.0;

            // Collect parents
            let parents = data.parents();
            let parent_info: Vec<_> = parents
                .keys()
                .filter_map(|parent_idx| {
                    graph_ref.get_vertex_data(*parent_idx).ok().map(|pdata| {
                        let pname = graph_ref.vertex_data_string(pdata.clone());
                        let pkey = pdata.vertex_key();
                        (pname, pkey)
                    })
                })
                .collect();

            // Collect children
            let patterns = data.child_patterns();
            let mut shown_children = std::collections::HashSet::new();
            let child_info: Vec<_> = patterns
                .iter()
                .flat_map(|(_pat_id, pattern)| pattern.iter())
                .filter_map(|child| {
                    let child_idx = child.index;
                    if shown_children.insert(child_idx) {
                        graph_ref.get_vertex_data(child_idx).ok().map(|cdata| {
                            let cname =
                                graph_ref.vertex_data_string(cdata.clone());
                            let ckey = cdata.vertex_key();
                            (cname, ckey)
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Some((name, width, parent_info, child_info, key))
        });

        // Track what was clicked
        let mut new_selection: Option<Option<VertexKey>> = None;

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(250.0)
            .show_animated(ctx, self.right_panel_open, |ui| {
                ui.heading("Properties");
                ui.separator();

                // Show graph info
                if let Some(count) = vertex_count {
                    ui.label(format!("Vertices: {}", count));
                }

                ui.add_space(10.0);
                ui.separator();
                ui.heading("Selection");

                if let Some((
                    name,
                    width,
                    parent_info,
                    child_info,
                    _selected_key,
                )) = &selection_info
                {
                    // Show mini neighborhood graph
                    ui.add_space(5.0);

                    let graph_height = 200.0;
                    let (response, painter) = ui.allocate_painter(
                        Vec2::new(ui.available_width(), graph_height),
                        egui::Sense::click(),
                    );
                    let rect = response.rect;

                    // Draw background
                    painter.rect_filled(
                        rect,
                        4.0,
                        Color32::from_rgb(30, 33, 38),
                    );
                    painter.rect_stroke(
                        rect,
                        4.0,
                        Stroke::new(1.0, Color32::from_rgb(60, 65, 75)),
                        egui::StrokeKind::Inside,
                    );

                    // Layout: parents at top, selected in middle, children at bottom
                    let center_x = rect.center().x;
                    let selected_y = rect.center().y;
                    let parent_y = rect.min.y + 35.0;
                    let child_y = rect.max.y - 35.0;

                    // Selected node (larger, in center)
                    let selected_size = Vec2::new(120.0, 40.0);
                    let selected_rect = Rect::from_center_size(
                        Pos2::new(center_x, selected_y),
                        selected_size,
                    );

                    // Draw selected node
                    painter.rect_filled(
                        selected_rect,
                        6.0,
                        Color32::from_rgb(50, 100, 150),
                    );
                    painter.rect_stroke(
                        selected_rect,
                        6.0,
                        Stroke::new(2.0, Color32::from_rgb(100, 150, 200)),
                        egui::StrokeKind::Inside,
                    );

                    // Truncate name if too long
                    let display_name = if name.len() > 14 {
                        format!("{}...", &name[..11])
                    } else {
                        name.clone()
                    };
                    painter.text(
                        selected_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &display_name,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );

                    // Draw parent nodes (small, at top)
                    let parent_count = parent_info.len();
                    let small_size = Vec2::new(60.0, 24.0);
                    let spacing = 70.0;
                    let parent_start_x =
                        center_x - (parent_count as f32 - 1.0) * spacing / 2.0;

                    for (i, (pname, pkey)) in parent_info.iter().enumerate() {
                        let px = parent_start_x + i as f32 * spacing;
                        let parent_rect = Rect::from_center_size(
                            Pos2::new(px, parent_y),
                            small_size,
                        );

                        // Check if clicked
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if parent_rect.contains(pos) {
                                    new_selection = Some(Some(*pkey));
                                }
                            }
                        }

                        let is_hovered = response.hovered()
                            && ui
                                .input(|i| i.pointer.hover_pos())
                                .map(|p| parent_rect.contains(p))
                                .unwrap_or(false);

                        let fill = if is_hovered {
                            Color32::from_rgb(70, 80, 90)
                        } else {
                            Color32::from_rgb(50, 55, 65)
                        };

                        painter.rect_filled(parent_rect, 4.0, fill);
                        painter.rect_stroke(
                            parent_rect,
                            4.0,
                            Stroke::new(1.0, Color32::from_rgb(80, 90, 100)),
                            egui::StrokeKind::Inside,
                        );

                        let short_name = if pname.len() > 6 {
                            format!("{}…", &pname[..5])
                        } else {
                            pname.clone()
                        };
                        painter.text(
                            parent_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &short_name,
                            egui::FontId::proportional(10.0),
                            Color32::LIGHT_GRAY,
                        );

                        // Draw edge from parent to selected (arrow pointing down)
                        let edge_start = Pos2::new(px, parent_rect.max.y);
                        let edge_end = Pos2::new(center_x, selected_rect.min.y);
                        Self::draw_edge(&painter, edge_start, edge_end);
                    }

                    // Draw child nodes (small, at bottom)
                    let child_count = child_info.len();
                    let child_start_x =
                        center_x - (child_count as f32 - 1.0) * spacing / 2.0;

                    for (i, (cname, ckey)) in child_info.iter().enumerate() {
                        let cx = child_start_x + i as f32 * spacing;
                        let child_rect = Rect::from_center_size(
                            Pos2::new(cx, child_y),
                            small_size,
                        );

                        // Check if clicked
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if child_rect.contains(pos) {
                                    new_selection = Some(Some(*ckey));
                                }
                            }
                        }

                        let is_hovered = response.hovered()
                            && ui
                                .input(|i| i.pointer.hover_pos())
                                .map(|p| child_rect.contains(p))
                                .unwrap_or(false);

                        let fill = if is_hovered {
                            Color32::from_rgb(70, 80, 90)
                        } else {
                            Color32::from_rgb(50, 55, 65)
                        };

                        painter.rect_filled(child_rect, 4.0, fill);
                        painter.rect_stroke(
                            child_rect,
                            4.0,
                            Stroke::new(1.0, Color32::from_rgb(80, 90, 100)),
                            egui::StrokeKind::Inside,
                        );

                        let short_name = if cname.len() > 6 {
                            format!("{}…", &cname[..5])
                        } else {
                            cname.clone()
                        };
                        painter.text(
                            child_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &short_name,
                            egui::FontId::proportional(10.0),
                            Color32::LIGHT_GRAY,
                        );

                        // Draw edge from selected to child (arrow pointing down)
                        let edge_start =
                            Pos2::new(center_x, selected_rect.max.y);
                        let edge_end = Pos2::new(cx, child_rect.min.y);
                        Self::draw_edge(&painter, edge_start, edge_end);
                    }

                    // Show "no parents" or "no children" text
                    if parent_info.is_empty() {
                        painter.text(
                            Pos2::new(center_x, parent_y),
                            egui::Align2::CENTER_CENTER,
                            "(no parents)",
                            egui::FontId::proportional(10.0),
                            Color32::DARK_GRAY,
                        );
                    }
                    if child_info.is_empty() {
                        painter.text(
                            Pos2::new(center_x, child_y),
                            egui::Align2::CENTER_CENTER,
                            "(no children)",
                            egui::FontId::proportional(10.0),
                            Color32::DARK_GRAY,
                        );
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // Show properties below the graph
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.strong(name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.label(format!("{}", width));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Parents:");
                        ui.label(format!("{}", parent_info.len()));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Children:");
                        ui.label(format!("{}", child_info.len()));
                    });
                } else if selected_node.is_some() {
                    ui.label("(Could not load selection)");
                } else {
                    ui.label("Click a node to select it");
                }
            });

        // Apply selection change after panel is done
        if let Some(sel) = new_selection {
            if let Some(tab) = self.current_tab_mut() {
                tab.selected_node = sel;
            }
        }
    }

    /// Draw an edge with an arrow
    fn draw_edge(
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
    ) {
        let color = Color32::from_rgb(120, 130, 140);
        painter.line_segment([start, end], Stroke::new(1.5, color));

        // Arrow head
        let dir = (end - start).normalized();
        let arrow_size = 6.0;
        let arrow_angle = std::f32::consts::PI / 6.0;

        let left = Pos2::new(
            end.x
                - arrow_size
                    * (dir.x * arrow_angle.cos() - dir.y * arrow_angle.sin()),
            end.y
                - arrow_size
                    * (dir.y * arrow_angle.cos() + dir.x * arrow_angle.sin()),
        );
        let right = Pos2::new(
            end.x
                - arrow_size
                    * (dir.x * arrow_angle.cos() + dir.y * arrow_angle.sin()),
            end.y
                - arrow_size
                    * (dir.y * arrow_angle.cos() - dir.x * arrow_angle.sin()),
        );

        painter.add(egui::Shape::convex_polygon(
            vec![end, left, right],
            color,
            Stroke::NONE,
        ));
    }

    pub(crate) fn bottom_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(150.0)
            .height_range(50.0..=400.0)
            .show_animated(ctx, self.bottom_panel_open, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Output");
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if ui.small_button("Clear").clicked() {
                                self.output.clear();
                            }
                        },
                    );
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let lines = self.output.lines();
                        if lines.is_empty() {
                            ui.colored_label(
                                egui::Color32::DARK_GRAY,
                                "(No output yet)",
                            );
                        } else {
                            for line in lines {
                                ui.horizontal(|ui| {
                                    ui.colored_label(
                                        line.level.color(),
                                        line.level.prefix(),
                                    );
                                    ui.label(&line.text);
                                });
                            }
                        }
                    });
            });
    }

    pub(crate) fn status_bar(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Show task status (unified for native and wasm)
                    if self.is_task_running() {
                        ui.spinner();
                        ui.label("Processing...");

                        // Show progress if available
                        if let Some(read_ctx) = self.ctx() {
                            if let Some(status) = read_ctx.status() {
                                let status = status.read().unwrap();
                                ui.separator();
                                ui.label(format!("Pass: {:?}", status.pass()));
                                let progress = *status.steps() as f32
                                    / *status.steps_total() as f32;
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .desired_width(150.0)
                                        .show_percentage(),
                                );
                            }
                        }
                    } else {
                        ui.label("Ready");
                    }

                    // Debug build warning on the right
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            egui::warn_if_debug_build(ui);
                        },
                    );
                });
            });
    }
}
