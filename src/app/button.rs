use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};

use crate::app::popup::{ItemStyle, PopupFocus};

pub struct Button {
    label: &'static str,
    style: ItemStyle,
}

impl Button {
    pub fn new(label: &'static str, style: ItemStyle) -> Self {
        Self { label, style }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if key_event.code == KeyCode::Enter {
            return true;
        };
        return false;
    }
}

impl StatefulWidget for &Button {
    type State = PopupFocus;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PopupFocus) {
        let label = Line::from(self.label).bold();
        let style = match state {
            PopupFocus::Ok => self.style.selected,
            _ => self.style.non_selected,
        };
        let button = Paragraph::new(label)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .style(style);
        button.render(area, buf);
    }
}
