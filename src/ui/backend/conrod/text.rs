use conrod::color;
use conrod::color::Colorable;
use conrod::widget;
use conrod::widget::TextEdit;
use conrod::UiCell;
use conrod::Widget;
use conrod::{Positionable, Sizeable};

use ui::TextView;

use super::Update;

pub struct Text {
    editable: bool,
    id: widget::Id,
    parent: widget::Id,
    text: String,
}

impl Text {
    pub fn new(widget_id: widget::Id, parent_id: widget::Id, editable: bool) -> Text {
        return Text {
            editable: editable,
            id: widget_id,
            parent: parent_id,
            text: String::new(),
        };
    }

    pub fn submit(&mut self) -> String {
        let ret = self.get_text().clone();
        self.set_text("");
        return ret;
    }
}

impl Update for Text {
    fn update(&mut self, ui_cell: &mut UiCell) {
        if self.editable {
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
        } else {
            widget::Text::new(self.text.as_str())
                .color(color::BLUE)
                .padded_w_of(self.parent, 16.0)
                .set(self.id, ui_cell)
        }
    }
}

impl TextView for Text {
    fn get_text(&self) -> &String {
        return &self.text;
    }

    fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
    }
}
