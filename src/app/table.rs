use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Direction;
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
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::app::color::{PALETTES, TableColors};
use crate::app::footer::{self, Footer};
use crate::app::popup::PopupForm;
use crate::labeling::FieldMap;
use crate::models::{Category, Transaction};

impl Transaction {
    fn ref_array(&self, index: u32) -> [Cow<str>; 6] {
        let group_string = match &self.group {
            Some(s) => s.to_string(),
            None => "N/A".to_string(),
        };
        let amount_string = format!("{:.2}", self.amount);
        [
            Cow::Owned(index.to_string()),
            Cow::Owned(self.date.to_string()),
            Cow::Borrowed(&self.title),
            Cow::Owned(amount_string),
            Cow::Owned(self.kind.to_string()),
            Cow::Owned(group_string),
        ]
    }
}

enum SortOptions {
    DateAsc,
    DateDesc,
    TitleAsc,
    TitleDesc,
    AmountAsc,
    AmountDesc,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TableMode {
    Popup,
    Normal,
    Ordering,
    Help,
    Search,
    Searched,
}

pub struct TableComponent {
    state: TableState,
    pub items: Vec<Transaction>,
    filtered_items: Vec<Transaction>,
    colors: TableColors,
    popup: PopupForm,
    mode: TableMode,
    footer: Footer,
    title_map: FieldMap<String>,
    category_map: FieldMap<Category>,
}

impl TableComponent {
    pub fn new(transactions: &Vec<Transaction>) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            colors: TableColors::new(&PALETTES[0]),
            items: transactions.to_vec(),
            filtered_items: transactions.to_vec(),
            mode: TableMode::Normal,
            title_map: FieldMap::<String>::new(),
            category_map: FieldMap::<Category>::new(),
            popup: PopupForm::new(transactions[0].clone()),
            footer: Footer::new(),
        }
    }

    pub fn set_categories(&mut self, category_map: FieldMap<Category>) {
        self.category_map = category_map;
    }

    pub fn get_categories(&self) -> FieldMap<Category> {
        self.category_map.clone()
    }

    pub fn set_titlemap(&mut self, title_map: FieldMap<String>) {
        self.title_map = title_map;
    }

    pub fn get_titlemap(&self) -> FieldMap<String> {
        self.title_map.clone()
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
        match self.mode {
            TableMode::Searched => {
                self.filtered_items[self.state.selected().expect("Line number")].clone()
            }
            _ => self.items[self.state.selected().expect("Line number")].clone(),
        }
    }

    pub fn set_current_row(&mut self, transaction: &Transaction) {
        self.category_map
            .insert(&transaction.title, &transaction.group);

        let mut item_title = "".to_string();
        for mut item in self.items.iter_mut() {
            if item.id == transaction.id {
                item_title = item.title.clone();
                item = &mut transaction.clone();
            }
        }

        for (key, value) in &self.title_map.map.clone() {
            if *value == item_title {
                self.title_map
                    .insert(&key.to_string(), &Some(transaction.title.clone()));
            }
        }

        self.title_map
            .insert(&item_title, &Some(transaction.title.clone()));
        self.update_transactions();
    }

    pub fn update_transactions(&mut self) {
        for transaction in self.items.iter_mut() {
            match self.title_map.get(&transaction.title) {
                Some(title) => {
                    transaction.title = title.clone();
                }
                None => (),
            }

            transaction.group = self.category_map.get(&transaction.title);
        }
    }

    fn search_items(&mut self, substring: String) {
        self.filtered_items = self
            .items
            .clone()
            .into_iter()
            .filter(|row| {
                row.title
                    .to_ascii_lowercase()
                    .contains(&substring.to_ascii_lowercase())
            })
            .collect();
        self.state.select_first();
    }

    fn sort_items(&mut self, sort_option: SortOptions) {
        let sort_closure: fn(&Transaction, &Transaction) -> Ordering = match sort_option {
            SortOptions::DateAsc => |a, b| a.date.cmp(&b.date),
            SortOptions::DateDesc => |a, b| b.date.cmp(&a.date),
            SortOptions::TitleAsc => |a, b| a.title.cmp(&b.title),
            SortOptions::TitleDesc => |a, b| b.title.cmp(&a.title),
            SortOptions::AmountAsc => |a, b| a.amount.partial_cmp(&b.amount).unwrap(),
            SortOptions::AmountDesc => |a, b| b.amount.partial_cmp(&a.amount).unwrap(),
        };
        self.items.sort_by(sort_closure);
        self.state.select_first();
    }

    pub fn is_blocking(&self) -> bool {
        self.mode != TableMode::Normal
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        match self.mode {
            TableMode::Normal => match key_event.code {
                KeyCode::Enter => {
                    self.mode = TableMode::Popup;
                    self.popup = PopupForm::new(self.get_current_row());
                }
                KeyCode::Char('k') | KeyCode::Down => self.next_row(),
                KeyCode::Char('l') | KeyCode::Up => self.previous_row(),
                KeyCode::Char('o') => self.mode = TableMode::Ordering,
                KeyCode::Char('?') => self.mode = TableMode::Help,
                KeyCode::Char('/') => {
                    self.filtered_items = self.items.clone();
                    self.mode = TableMode::Search;
                }
                _ => (),
            },
            TableMode::Popup => match self.popup.handle_key_event(key_event) {
                Some(transaction) => {
                    self.set_current_row(&transaction);
                    self.mode = TableMode::Normal;
                }
                None => (),
            },
            TableMode::Ordering => {
                match key_event.code {
                    KeyCode::Char('d') => self.sort_items(SortOptions::DateAsc),
                    KeyCode::Char('D') => self.sort_items(SortOptions::DateDesc),
                    KeyCode::Char('t') => self.sort_items(SortOptions::TitleAsc),
                    KeyCode::Char('T') => self.sort_items(SortOptions::TitleDesc),
                    KeyCode::Char('a') => self.sort_items(SortOptions::AmountAsc),
                    KeyCode::Char('A') => self.sort_items(SortOptions::AmountDesc),
                    _ => (),
                }
                self.mode = TableMode::Normal;
            }
            TableMode::Search => {
                self.state.select_first();
                match key_event.code {
                    KeyCode::Enter => {
                        self.mode = TableMode::Searched;
                        self.footer.search().clear_value();
                    }
                    KeyCode::Esc => {
                        self.mode = TableMode::Normal;
                        self.footer.search().clear_value();
                    }
                    _ => {
                        self.footer.search().handle_key_event(key_event);
                        let substring = self.footer.search().get_value();
                        self.search_items(substring);
                    }
                };
            }
            TableMode::Searched => match key_event.code {
                KeyCode::Esc => {
                    self.mode = TableMode::Normal;
                }
                KeyCode::Enter => {
                    self.popup = PopupForm::new(self.get_current_row());
                    self.mode = TableMode::Popup;
                }
                KeyCode::Char('k') | KeyCode::Down => self.next_row(),
                KeyCode::Char('l') | KeyCode::Up => self.previous_row(),
                _ => (),
            },
            TableMode::Help => self.mode = TableMode::Normal,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let footer_size = match self.mode {
            TableMode::Normal => 0,
            TableMode::Ordering => 6 + 2,
            TableMode::Popup => 0,
            TableMode::Help => 5 + 2,
            TableMode::Search => 1,
            TableMode::Searched => 1,
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(footer_size)])
            .split(area);

        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);

        let header = ["", "Date", "Title", "Amount (R$)", "Kind", "Group"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let item_list = match self.mode {
            TableMode::Searched => &self.filtered_items,
            TableMode::Search => &self.filtered_items,
            _ => &self.items,
        };

        let rows = item_list.iter().enumerate().map(|(i, data)| {
            let item = data.ref_array(i as u32);
            item.into_iter().map(Cell::from).collect::<Row>().height(1)
        });
        let bar = " █ ";
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Transactions");
        let t = Table::new(
            rows,
            [
                Constraint::Max(5),
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

        frame.render_stateful_widget(t, layout[0], &mut self.state);
        frame.render_stateful_widget(&self.footer, layout[1], &mut self.mode);

        match self.mode {
            TableMode::Popup => {
                self.popup.render(frame);
            }
            _ => (),
        }
    }
}
