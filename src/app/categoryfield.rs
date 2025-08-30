use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

use crate::app::popup::{ItemStyle, PopupFocus};
use crate::models::Category;

pub struct CategoryField {
    label: &'static str,
    selected: Category,
    style: ItemStyle,
}

impl CategoryField {
    pub fn new(label: &'static str, value: Category, style: ItemStyle) -> Self {
        Self {
            label,
            selected: value,
            style,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Right | KeyCode::Char(';') => self.next(),
            KeyCode::Left | KeyCode::Char('j') => self.previous(),
            _ => (),
        }
    }

    fn next(&mut self) {
        self.selected = self.selected.next();
    }

    fn previous(&mut self) {
        self.selected = self.selected.previous();
    }

    pub fn value(&self) -> Category {
        self.selected.clone()
    }
}

impl StatefulWidget for &CategoryField {
    type State = PopupFocus;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PopupFocus) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Max(self.value().to_string().len() as u16 + 4),
        ])
        .split(area);

        let label = Line::from_iter([self.label, ": "]).bold();
        label.render(layout[0], buf);
        let value = format!("< {} >", self.value());

        let style = match state {
            PopupFocus::Category => self.style.selected,
            _ => self.style.non_selected,
        };
        Paragraph::new(value)
            .clone()
            .style(style)
            .render(layout[1], buf);
    }
}
