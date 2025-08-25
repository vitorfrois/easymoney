use ratatui::style::palette::tailwind;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget},
};

pub const INFO_TEXT: [&str; 1] = ["(Esc) quit | (↑) move up | (↓) move down"];

use crate::app::color::{PALETTES, TableColors};

pub struct Footer {
    colors: TableColors,
}

impl Footer {
    pub fn new() -> Self {
        Footer {
            colors: TableColors::new(&PALETTES[0]),
        }
    }
}

impl Widget for &Footer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        // frame.render_widget(info_footer, area);
        info_footer.render(area, buf);
    }
}
