//! Menu bar and context menu logic.

use eframe::egui::{
    self,
    Ui,
};
use strum::IntoEnumIterator;

use super::App;
use crate::{
    algorithm::Algorithm,
    examples::{
        build_graph1,
        build_graph2,
        build_graph3,
    },
};

impl App {
    pub fn context_menu(
        &mut self,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            ui.label("Quick Insert:");
            if let Some(mut ctx) = self.ctx_mut() {
                for text in &mut ctx.graph_mut().insert_texts {
                    ui.text_edit_singleline(text);
                }
            }
            if ui.button("Go").clicked() {
                self.start_read();
                ui.close();
            }
            if ui.button("Cancel").clicked() {
                self.abort();
            }
        });

        if ui.button("Toggle Inserter").clicked() {
            self.inserter_open = !self.inserter_open;
            ui.close();
        }

        ui.menu_button("Load preset...", |ui| {
            if let Some(ctx) = self.ctx() {
                if ui.button("Graph 1").clicked() {
                    ctx.graph().set_graph(build_graph1());
                    ui.close();
                }
                if ui.button("Graph 2").clicked() {
                    ctx.graph().set_graph(build_graph2());
                    ui.close();
                }
                if ui.button("Graph 3").clicked() {
                    ctx.graph().set_graph(build_graph3());
                    ui.close();
                }
            }
        });

        if ui.button("Clear").clicked() {
            if let Some(mut ctx) = self.ctx_mut() {
                ctx.graph_mut().clear();
            }
            if let Some(mut vis) = self.vis_mut() {
                vis.mark_dirty();
            }
            ui.close();
        }
    }

    pub(crate) fn show_settings_window(
        &mut self,
        ctx: &egui::Context,
    ) {
        if !self.settings_open {
            return;
        }

        egui::Window::new("Settings")
            .open(&mut self.settings_open)
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Algorithm Selection");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Algorithm:");
                    egui::ComboBox::from_id_salt("algorithm_selector")
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
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                // Show algorithm description
                ui.label("Description:");
                ui.label(self.selected_algorithm.description());

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(5.0);

                ui.heading("Panel Layout");
                ui.add_space(10.0);

                ui.checkbox(
                    &mut self.bottom_panel_overlaps_left,
                    "Bottom panel overlaps left sidebar",
                );
                ui.checkbox(
                    &mut self.bottom_panel_overlaps_right,
                    "Bottom panel overlaps right sidebar",
                );
            });
    }
}
