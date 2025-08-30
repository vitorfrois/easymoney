use std::vec::Vec;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Stylize;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Cell, Row, StatefulWidget, Table, TableState};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Paragraph},
};

use crate::app::color::{PALETTES, TableColors};
use crate::app::popup::{ItemStyle, PopupFocus};
use crate::app::stringfield::StringField;
use crate::app::table::TableMode;

struct FooterInfo<'a> {
    command_map: Vec<(&'a str, &'a str)>,
}

impl FooterInfo<'_> {
    fn get_command(&self) -> Vec<(&'_ str, &'_ str)> {
        self.command_map.clone()
    }

    fn get_table(&self) -> Table<'_> {
        let rows: Vec<Row> = self
            .get_command()
            .into_iter()
            .map(|data| Row::new([data.0, data.1]))
            .collect();

        let table = Table::new(rows, [Constraint::Max(15), Constraint::Fill(1)]);
        table
    }
}

fn get_ordering() -> FooterInfo<'static> {
    let command_map = Vec::from([
        ("D", "Date Desc"),
        ("d", "Date asc"),
        ("T", "Title Desc"),
        ("t", "Title asc"),
        ("A", "Amount Desc"),
        ("a", "Amount asc"),
    ]);
    FooterInfo { command_map }
}

fn get_help() -> FooterInfo<'static> {
    let command_map = Vec::from([
        ("Enter", "Edit Transaction"),
        ("o", "Order By"),
        ("↑", "Move Up"),
        ("↓", "Move Down"),
        ("q", "Quit"),
    ]);
    FooterInfo { command_map }
}

pub struct Footer {
    colors: TableColors,
    search: StringField,
}

impl Footer {
    pub fn new() -> Self {
        let colors = TableColors::new(&PALETTES[0]);
        let item_style = ItemStyle::new(&colors);

        Footer {
            colors: TableColors::new(&PALETTES[0]),
            search: StringField::new("/", &"".to_string(), 40, item_style),
        }
    }

    pub fn search(&mut self) -> &mut StringField {
        &mut self.search
    }
}

impl StatefulWidget for &Footer {
    type State = TableMode;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut TableMode) {
        let style = Style::new()
            .fg(self.colors.header_fg)
            .bg(self.colors.buffer_bg);

        let table = match state {
            TableMode::Ordering => Some(get_ordering()),
            TableMode::Help => Some(get_help()),
            TableMode::Search => {
                &self.search.render(area, buf, &mut PopupFocus::Title);
                None
            }
            _ => None,
        };

        let title = match state {
            TableMode::Ordering => "Order By",
            _ => "",
        };

        let block = Block::bordered()
            .title(title)
            .border_type(BorderType::Rounded)
            .border_style(style);

        match table {
            Some(table) => table.get_table().style(style).block(block).render(
                area,
                buf,
                &mut TableState::new(),
            ),
            None => (),
        };
    }
}
