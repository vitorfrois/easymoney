use ratatui::style::Stylize;
use ratatui::style::palette::tailwind;
use ratatui::widgets::StatefulWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget},
};

const INFO_TEXT: [&str; 1] = ["(Esc) quit | (↑) move up | (↓) move down"];
const ORD_TEXT: [&str; 7] = [
    "Order by",
    "[D]: Date Desc",
    "[d]: Date asc",
    "[T]: Title Desc",
    "[t]: Title asc",
    "[A]: Amount Desc",
    "[a]: Amount asc",
];

use crate::app::color::{PALETTES, TableColors};
use crate::app::table::TableMode;

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

impl StatefulWidget for &Footer {
    type State = TableMode;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut TableMode) {
        let style = Style::new()
            .fg(self.colors.header_fg)
            .bg(self.colors.buffer_bg);

        let text = match state {
            TableMode::Ordering => Paragraph::new(Text::from_iter(ORD_TEXT))
                .style(style)
                .left_aligned(),
            _ => Paragraph::new(Text::from_iter(INFO_TEXT))
                .style(style)
                .centered(),
        };

        let info_footer = text.block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(self.colors.footer_border_color))
                .bg(self.colors.buffer_bg),
        );
        info_footer.render(area, buf);
    }
}
