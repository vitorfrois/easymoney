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
use strum::{Display, FromRepr, IntoEnumIterator};
use strum_macros::EnumIter;
use style::palette::tailwind;

use crate::app::table::TableComponent;
use crate::models::Transaction;

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum CurrentTab {
    #[default]
    #[strum(to_string = "Transactions")]
    Table,
    #[strum(to_string = "Home")]
    Home,
}

impl CurrentTab {
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

struct App {
    items: Vec<Transaction>,
    current_tab: CurrentTab,
    table: TableComponent,
    edit_popup: bool,
}

impl App {
    fn new(transactions: Vec<Transaction>) -> Self {
        Self {
            table: TableComponent::new(&transactions),
            current_tab: CurrentTab::Table,
            items: transactions,
            edit_popup: false,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
                        KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
                        keycode => match self.current_tab {
                            CurrentTab::Home => (),
                            CurrentTab::Table => self.table.run(terminal, &keycode),
                        },
                    }
                }
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(frame.area());

        match self.current_tab {
            CurrentTab::Table => self.table.draw(frame),
            CurrentTab::Home => (),
        }
    }
}

pub fn init_app(transactions: Vec<Transaction>) -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(transactions).run(terminal);
    ratatui::restore();
    app_result
}
