use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::style::palette::tailwind;
use ratatui::widgets::StatefulWidget;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Table, TableState},
};
use std::borrow::Cow;

use crate::app::color::{PALETTES, TableColors};
use crate::app::footer::Footer;
use crate::app::popup::PopupForm;
use crate::models::Transaction;

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
    focus_popup: bool,
    popup: PopupForm,
    footer: Footer,
}

impl TableComponent {
    pub fn new(transactions: &Vec<Transaction>) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            colors: TableColors::new(&PALETTES[0]),
            items: transactions.to_vec(),
            focus_popup: false,
            popup: PopupForm::new(transactions[0].clone()),
            footer: Footer::new(),
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

    pub fn get_current_row(&self) -> Transaction {
        self.items[self.state.selected().expect("Line number")].clone()
    }

    pub fn set_current_row(&mut self, transaction: Transaction) {
        self.items[self.state.selected().expect("Line number")] = transaction;
    }

    pub fn is_blocking(&self) -> bool {
        self.focus_popup
    }

    pub fn toggle_popup(&mut self) {
        self.focus_popup = !self.focus_popup;
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        if self.focus_popup {
            match self.popup.handle_key_event(key_event) {
                Some(transaction) => {
                    self.set_current_row(transaction);
                    self.toggle_popup();
                }
                None => (),
            }
        } else {
            match key_event.code {
                KeyCode::Enter => {
                    self.toggle_popup();
                    self.popup = PopupForm::new(self.get_current_row());
                }
                KeyCode::Char('k') | KeyCode::Down => self.next_row(),
                KeyCode::Char('l') | KeyCode::Up => self.previous_row(),
                _ => (),
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
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

        if self.focus_popup {
            self.popup.render(frame);
        }
    }
}
