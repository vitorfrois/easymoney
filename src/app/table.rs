use color_eyre::Result;
use itertools::Itertools;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use std::borrow::Cow;
use style::palette::tailwind;

use crate::app::popup::PopupForm;
use crate::models::Transaction;

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

const INFO_TEXT: [&str; 1] = ["(Esc) quit | (↑) move up | (↓) move down"];

const ITEM_HEIGHT: usize = 1;

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

impl Transaction {
    fn ref_array(&self) -> [Cow<str>; 5] {
        let group_string = match &self.group {
            Some(s) => s.to_string(),
            None => "N/A".to_string(),
        };
        let amount_string = format!("{:.2}", self.amount);
        [
            Cow::Owned(self.date.to_string()),
            Cow::Borrowed(&self.title),
            Cow::Owned(amount_string),
            Cow::Owned(self.kind.to_string()),
            Cow::Owned(group_string),
        ]
    }
}

pub struct TableComponent {
    state: TableState,
    items: Vec<Transaction>,
    colors: TableColors,
    edit_popup: bool,
    popup: PopupForm,
}

impl TableComponent {
    pub fn new(transactions: &Vec<Transaction>) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            colors: TableColors::new(&PALETTES[0]),
            items: transactions.to_vec(),
            edit_popup: false,
            popup: PopupForm::new(&transactions[0]),
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_current_row(&mut self) -> &Transaction {
        &self.items[self.state.selected().expect("Line number")]
    }

    pub fn set_current_row(&mut self, transaction: Transaction) {
        self.items[self.state.selected().expect("Line number")] = transaction;
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal, keycode: &KeyCode) {
        if self.edit_popup {
            match self.popup.run(terminal) {
                Some(transaction) => self.set_current_row(transaction),
                None => (),
            }
        } else {
            match keycode {
                KeyCode::Enter => {
                    self.edit_popup = !self.edit_popup;
                    self.popup = PopupForm::new(self.get_current_row());
                }
                KeyCode::Char('k') | KeyCode::Down => self.next_row(),
                KeyCode::Char('l') | KeyCode::Up => self.previous_row(),
                _ => {}
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(frame.area());

        self.render_table(frame, rects[0]);
        // self.render_scrollbar(frame, rects[0]);
        self.render_footer(frame, rects[1]);
        if self.edit_popup {
            self.popup.render(frame);
        }
    }

    pub fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        // let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        // let selected_cell_style = Style::default()
        //     .add_modifier(Modifier::REVERSED)
        //     .fg(self.colors.selected_cell_style_fg);

        let header = ["Date", "Title", "Amount (R$)", "Kind", "Group"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(Cell::from)
                .collect::<Row>()
                // .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(1)
        });
        let bar = " █ ";
        let block = Block::bordered().title("Transactions");
        let t = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Min(20),
                Constraint::Length(12),
                Constraint::Length(20),
                Constraint::Length(20),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        // .column_highlight_style(selected_col_style)
        // .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always)
        .block(block);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
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
        frame.render_widget(info_footer, area);
    }
}
