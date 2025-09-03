use std::fmt::format;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Tabs, Widget};
use ratatui::{DefaultTerminal, Frame};
use strum::{Display, FromRepr, IntoEnumIterator};
use strum_macros::EnumIter;

use crate::app::chart::ChartComponent;
use crate::db;
use crate::event::{AppEvent, EventHandler};
use crate::labeling;
use crate::models::{Category, NewTransaction, Transaction};
use crate::{app::table::TableComponent, db::Database};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum CurrentTab {
    #[default]
    #[strum(to_string = "Transactions")]
    Table,
    #[strum(to_string = "Chart")]
    Chart,
}

impl CurrentTab {
    fn title(self) -> Line<'static> {
        format!("  {self}  ").into()
    }
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
    database: Database,
    running: bool,
    has_changed: bool,
    events: EventHandler,
    items: Vec<Transaction>,
    current_tab: CurrentTab,
    pub table: TableComponent,
    pub chart: ChartComponent,
}

impl App {
    fn new(transactions: Vec<NewTransaction>) -> Self {
        let database = db::Database::new().expect("Could not acess Database");
        match database.insert_transactions(transactions) {
            Ok(v) => v,
            Err(_) => println!("SQL Insert Error"),
        }
        let mut transactions = database.get_transactions().expect("Could not acess DB");
        let category_map = labeling::FieldMap::<Category>::new();
        for transaction in transactions.iter_mut() {
            transaction.group = category_map.get(&transaction.title);
        }

        let category_map = database.get_categories().expect("Could not acess DB");
        let title_map = database.get_titlemaps().expect("Could not acess DB");

        let mut app = Self {
            database,
            running: true,
            has_changed: true,
            events: EventHandler::new(),
            table: TableComponent::new(&transactions.to_vec()),
            current_tab: CurrentTab::Table,
            items: transactions.to_vec(),
            chart: ChartComponent::new(&transactions.to_vec()),
        };

        app.table.set_titlemap(title_map);
        app.table.set_categories(category_map);
        app.table.update_transactions();
        app
    }

    pub fn save_to_db(&self) {
        let _ = self.database.insert_categories(self.table.get_categories());
        let _ = self.database.insert_titlemaps(self.table.get_titlemap());
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<Vec<Transaction>> {
        while self.running {
            match self.events.next().await? {
                AppEvent::Tick => {
                    if self.has_changed {
                        terminal.draw(|frame| self.draw(frame))?;
                        self.has_changed = false;
                        self.chart.update_chart(&self.table.items);
                    }
                }
                AppEvent::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key) => {
                        self.handle_key_events(key)?;
                        self.has_changed = true;
                    }
                    _ => (),
                },
                AppEvent::Quit => self.quit(),
            }
        }

        self.save_to_db();

        Ok(self.table.items)
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.table.is_blocking() {
            self.table.handle_key_events(key_event);
            return Ok(());
        }

        match key_event.code {
            KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char(';') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('j') | KeyCode::Left => self.previous_tab(),
            _ => match self.current_tab {
                CurrentTab::Chart => self.chart.handle_key_events(key_event),
                CurrentTab::Table => self.table.handle_key_events(key_event),
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

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = CurrentTab::iter().map(CurrentTab::title);
        let selected_tab_index = self.current_tab as usize;
        Tabs::new(titles)
            .select(selected_tab_index)
            .divider(" ")
            .render(area, buf);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header_area, inner_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());

        self.render_tabs(header_area, frame.buffer_mut());

        match self.current_tab {
            CurrentTab::Table => self.table.render(frame, inner_area),
            CurrentTab::Chart => self.chart.render(frame, inner_area),
        }
    }
}

pub async fn init_app(transactions: Vec<NewTransaction>) -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(transactions);
    let _transactions = app.run(terminal).await;
    ratatui::restore();
    Ok(())
}
