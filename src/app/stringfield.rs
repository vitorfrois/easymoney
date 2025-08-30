use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Offset, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

use crate::app::popup::{ItemStyle, PopupFocus};

pub struct StringField {
    label: &'static str,
    value: String,
    max_length: usize,
    style: ItemStyle,
}

impl StringField {
    pub fn new(label: &'static str, value: &String, max_length: usize, style: ItemStyle) -> Self {
        Self {
            label,
            value: value.clone(),
            max_length,
            style,
        }
    }

    pub fn clear_value(&mut self) {
        self.value = "".to_string();
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                if self.value.len() < self.max_length {
                    self.value.push(c);
                }
            }
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => (),
        }
    }

    pub fn cursor_offset(&self) -> Offset {
        let x = (self.label.len() + self.value.len() + 2) as i32;
        Offset { x: x, y: 0 }
    }
}

impl StatefulWidget for &StringField {
    type State = PopupFocus;
    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut PopupFocus) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16),
            Constraint::Fill(1),
        ])
        .split(area);
        let label = Line::from(self.label).bold();

        label.render(layout[0], buf);
        let text = Paragraph::new(self.value.clone()).style(self.style.non_selected);

        text.render(layout[1], buf);
    }
}
