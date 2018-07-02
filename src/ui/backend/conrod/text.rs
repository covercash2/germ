use conrod::color;
use conrod::color::Colorable;
use conrod::widget;
use conrod::widget::TextEdit;
use conrod::UiCell;
use conrod::Widget;
use conrod::{Positionable, Sizeable};

use ui;

pub struct TextInput {
    parent: widget::Id,
    id: widget::Id,
    text: String,
}

impl TextInput {
    pub fn new(widget_id: widget::Id, parent_id: widget::Id) -> TextInput {
        return TextInput {
            parent: parent_id,
            id: widget_id,
            text: String::new(),
        };
    }

    pub fn update(&mut self, ui_cell: &mut UiCell) {
        match TextEdit::new(self.text.as_str())
            .color(color::WHITE)
            .padded_w_of(self.parent, 16.0)
            .mid_top_of(self.parent)
            .left_justify()
            .line_spacing(2.5)
            .restrict_to_height(false)
            .set(self.id, ui_cell)
        {
            Some(edited) => self.text = edited,
            None => (),
        }
    }
}

impl ui::TextInput for TextInput {
    fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
    }

    fn submit(&self) -> String {
        return String::from("not implemented");
    }
}
