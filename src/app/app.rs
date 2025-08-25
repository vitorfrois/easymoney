use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::{self, Buffer},
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use strum::{Display, FromRepr};
use strum_macros::EnumIter;

use crate::app::table::TableComponent;
use crate::event::{AppEvent, EventHandler};
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

pub struct App {
    running: bool,
    counter: u8,
    events: EventHandler,
    items: Vec<Transaction>,
    current_tab: CurrentTab,
    table: TableComponent,
    edit_popup: bool,
}

impl App {
    fn new(transactions: Vec<Transaction>) -> Self {
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            table: TableComponent::new(&transactions),
            current_tab: CurrentTab::Table,
            items: transactions,
            edit_popup: false,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            match self.events.next().await? {
                AppEvent::Tick => {
                    terminal.draw(|frame| self.draw(frame))?;
                }
                AppEvent::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key) => self.handle_key_events(key)?,
                    _ => (),
                },
                AppEvent::Quit => self.quit(),
            }
        }

        Ok(())
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.events.send(AppEvent::Quit),
            KeyCode::Char(';') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('j') | KeyCode::Left => self.previous_tab(),
            keycode => match self.current_tab {
                CurrentTab::Home => (),
                CurrentTab::Table => self.table.handle_key_events(keycode),
            },
        };
        Ok(())
    }

    fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_tab {
            CurrentTab::Table => self.table.render(frame, frame.area()),
            CurrentTab::Home => (),
        }
    }
}

pub async fn init_app(transactions: Vec<Transaction>) -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(transactions).run(terminal).await;
    ratatui::restore();
    app_result
}
