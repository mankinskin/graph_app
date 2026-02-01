//! Panel rendering (top, left, right, bottom).

use context_trace::graph::vertex::has_vertex_key::HasVertexKey;
use context_trace::VertexSet;
use eframe::egui;
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

                // View menu
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.left_panel_open, "Left Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.right_panel_open, "Right Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.bottom_panel_open, "Bottom Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.status_bar_open, "Status Bar")
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.separator();
                    if ui
                        .checkbox(&mut self.inserter_open, "Inserter Window")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.settings_open, "Settings Window")
                        .clicked()
                    {
                        ui.close();
                    }
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
                    if ui.button("▶ Run").clicked() && self.read_task.is_none()
                    {
                        self.start_read();
                    }
                    if self.read_task.is_some()
                        && ui.button("⏹ Cancel").clicked()
                    {
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
        let vertex_count = self.ctx()
            .and_then(|ctx| ctx.graph().try_read().map(|g| g.vertex_count()));
        
        // Collect selection info
        let selection_info = selected_node.and_then(|key| {
            let read_ctx = self.ctx()?;
            let graph = read_ctx.graph();
            let graph_ref = graph.try_read()?;
            let data = graph_ref.get_vertex_data(key).ok()?;
            
            let name = graph_ref.vertex_data_string(data.clone());
            let width = data.to_child().width.0;
            
            // Collect parents
            let parents = data.parents();
            let parent_info: Vec<_> = parents.keys()
                .filter_map(|parent_idx| {
                    graph_ref.get_vertex_data(*parent_idx).ok().map(|pdata| {
                        let name = graph_ref.vertex_data_string(pdata.clone());
                        let key = pdata.vertex_key();
                        (name, key)
                    })
                })
                .collect();
            
            // Collect children
            let patterns = data.child_patterns();
            let mut shown_children = std::collections::HashSet::new();
            let child_info: Vec<_> = patterns.iter()
                .flat_map(|(_pat_id, pattern)| pattern.iter())
                .filter_map(|child| {
                    let child_idx = child.index;
                    if shown_children.insert(child_idx) {
                        graph_ref.get_vertex_data(child_idx).ok().map(|cdata| {
                            let cname = graph_ref.vertex_data_string(cdata.clone());
                            let ckey = cdata.vertex_key();
                            (cname, ckey)
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            Some((name, width, parent_info, child_info))
        });
        
        // Track what was clicked
        let mut new_selection: Option<Option<context_trace::graph::vertex::key::VertexKey>> = None;
        
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.0)
            .min_width(200.0)
            .show_animated(ctx, self.right_panel_open, |ui| {
                ui.heading("Properties");
                ui.separator();

                // Show graph info
                if let Some(count) = vertex_count {
                    ui.label(format!("Vertices: {}", count));
                }

                ui.add_space(20.0);
                ui.separator();
                ui.heading("Selection");
                
                if let Some((name, width, parent_info, child_info)) = &selection_info {
                    ui.strong(name);
                    
                    ui.add_space(10.0);
                    
                    // Show properties
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.label(format!("{}", width));
                    });
                    
                    // Show parents
                    ui.add_space(10.0);
                    ui.label("Parents:");
                    if parent_info.is_empty() {
                        ui.label("  (none)");
                    } else {
                        for (parent_name, parent_key) in parent_info {
                            if ui.link(parent_name).clicked() {
                                new_selection = Some(Some(*parent_key));
                            }
                        }
                    }
                    
                    // Show children
                    ui.add_space(10.0);
                    ui.label("Children:");
                    if child_info.is_empty() {
                        ui.label("  (none)");
                    } else {
                        for (child_name, child_key) in child_info {
                            if ui.link(child_name).clicked() {
                                new_selection = Some(Some(*child_key));
                            }
                        }
                    }
                    
                    ui.add_space(10.0);
                    if ui.button("Clear Selection").clicked() {
                        new_selection = Some(None);
                    }
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
                    // Show task status
                    if self.read_task.is_some() {
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
