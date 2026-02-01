//! A label that can be edited by double-clicking.

use eframe::egui::{
    self,
    Response,
    Ui,
    Widget,
};

/// Response from an editable label interaction.
pub struct EditableLabelResponse {
    /// The label was clicked (single click).
    pub clicked: bool,
    /// The label was renamed (new name returned).
    pub renamed: Option<String>,
}

/// State for an editable label, stored externally.
#[derive(Debug, Default, Clone)]
pub struct EditableLabelState {
    pub editing: bool,
    pub buffer: String,
    /// True on the first frame of editing, used to request focus once.
    first_frame: bool,
}

impl EditableLabelState {
    pub fn start_editing(
        &mut self,
        current_text: &str,
    ) {
        self.editing = true;
        self.first_frame = true;
        self.buffer = current_text.to_string();
    }

    pub fn cancel(&mut self) {
        self.editing = false;
        self.first_frame = false;
        self.buffer.clear();
    }

    pub fn finish(&mut self) -> Option<String> {
        if self.editing {
            self.editing = false;
            self.first_frame = false;
            let result = if self.buffer.trim().is_empty() {
                None
            } else {
                Some(std::mem::take(&mut self.buffer))
            };
            self.buffer.clear();
            result
        } else {
            None
        }
    }
}

/// A label widget that shows a text edit when double-clicked.
pub struct EditableLabel<'a> {
    text: &'a str,
    state: &'a mut EditableLabelState,
    selected: bool,
    prefix: Option<&'a str>,
}

impl<'a> EditableLabel<'a> {
    pub fn new(
        text: &'a str,
        state: &'a mut EditableLabelState,
    ) -> Self {
        Self {
            text,
            state,
            selected: false,
            prefix: None,
        }
    }

    /// Mark this label as selected (highlighted).
    pub fn selected(
        mut self,
        selected: bool,
    ) -> Self {
        self.selected = selected;
        self
    }

    /// Add a prefix (e.g., an icon) before the text.
    pub fn prefix(
        mut self,
        prefix: &'a str,
    ) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
    ) -> EditableLabelResponse {
        let mut clicked = false;
        let mut renamed = None;

        if self.state.editing {
            // Calculate width based on current buffer content (grows as user types)
            let font_id = egui::TextStyle::Body.resolve(ui.style());
            let galley = ui.painter().layout_no_wrap(
                self.state.buffer.clone(),
                font_id,
                egui::Color32::WHITE,
            );
            let text_width = galley.size().x.max(50.0) + 16.0; // min width + padding

            let response = ui.add_sized(
                egui::vec2(text_width, ui.spacing().interact_size.y),
                egui::TextEdit::singleline(&mut self.state.buffer),
            );

            // Auto-focus only on the first frame of editing, and set cursor to end
            if self.state.first_frame {
                self.state.first_frame = false;
                response.request_focus();
                // Set cursor to end of text
                if let Some(mut state) =
                    egui::TextEdit::load_state(ui.ctx(), response.id)
                {
                    let ccursor =
                        egui::text::CCursor::new(self.state.buffer.len());
                    state.cursor.set_char_range(Some(
                        egui::text::CCursorRange::one(ccursor),
                    ));
                    state.store(ui.ctx(), response.id);
                }
            }

            // Enter confirms the edit
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                renamed = self.state.finish();
            }
            // Escape or clicking outside cancels
            else if response.lost_focus()
                || ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
                self.state.cancel();
            }
        } else {
            let label_text = match self.prefix {
                Some(prefix) => format!("{} {}", prefix, self.text),
                None => self.text.to_string(),
            };

            let response = ui.selectable_label(self.selected, label_text);

            if response.clicked() {
                clicked = true;
            }

            if response.double_clicked() {
                self.state.start_editing(self.text);
            }
        }

        EditableLabelResponse { clicked, renamed }
    }
}

impl<'a> Widget for EditableLabel<'a> {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        // For Widget trait, we just show and return a dummy response
        // Use show() method for full functionality
        let _ = self.show(ui);
        ui.label("") // Placeholder response
    }
}
