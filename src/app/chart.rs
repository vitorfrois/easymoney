use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, List, ListItem, ListState,
        StatefulWidget, Widget,
    },
};
use strum::{Display, IntoEnumIterator};

use crate::models::{self, Category, Kind, Transaction};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

const NULL_KEY: &str = "None";

fn vertical_barchart(category_totals: &HashMap<String, f64>) -> BarChart {
    let bars: Vec<Bar> = category_totals
        .iter()
        .sorted_by(|a, b| a.0.cmp(&b.0))
        .map(|(category, value)| vertical_bar(category.clone(), value))
        .collect();

    let title = Line::from("Finances").centered();
    let terminal_width = termsize::get().unwrap().cols;

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .block(Block::new().title(title))
        .bar_width((terminal_width - 10) / Category::iter().len() as u16)
        .bar_gap(2)
}

fn vertical_bar(category: String, value: &f64) -> Bar {
    Bar::default()
        .value(*value as u64)
        .label(Line::from(category))
}

fn get_transactions_by_category(transactions: &Vec<&models::Transaction>) -> HashMap<String, f64> {
    let mut category_totals: HashMap<String, f64> = Category::iter()
        .map(|category| (category.to_string(), 0.0))
        .collect();

    category_totals.insert(NULL_KEY.to_string(), 0.0);

    for transaction in transactions {
        let key = match &transaction.group {
            Some(category) => category.to_string(),
            None => NULL_KEY.to_string(),
        };
        *category_totals.entry(key).or_default() += &transaction.amount;
    }
    category_totals
}

fn get_transactions_by_month(transactions: &Vec<models::Transaction>) -> Vec<MonthExpense> {
    let month_set: HashSet<(i32, u32)> = HashSet::from(
        transactions
            .iter()
            .map(|row| (row.date.year(), row.date.month()))
            .collect::<HashSet<(i32, u32)>>(),
    );

    let month_expenses = month_set
        .iter()
        .map(|(year, month)| MonthExpense::new(transactions, *year, *month))
        .sorted_by(|a, b| (a.month as i32 + a.year).cmp(&(b.month as i32 + b.year)))
        .rev()
        .collect();

    month_expenses
}

#[derive(PartialEq, Debug)]
struct MonthExpense {
    year: i32,
    month: u32,
    total: f64,
    categorized: HashMap<String, f64>,
}

impl MonthExpense {
    fn new(transactions: &Vec<Transaction>, year: i32, month: u32) -> Self {
        let filtered_transactions: Vec<&models::Transaction> = transactions
            .iter()
            .filter(|row| row.date.month() == month && row.date.year() == year)
            .collect();
        let total = filtered_transactions.iter().map(|row| row.amount).sum();
        let categorized = get_transactions_by_category(&filtered_transactions);
        Self {
            year,
            month,
            total,
            categorized,
        }
    }
}

pub struct ChartComponent {
    items: Vec<MonthExpense>,
    state: ListState,
    max_height: f64,
}

impl ChartComponent {
    pub fn new(transactions: &Vec<models::Transaction>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        let mut chart_component = Self {
            items: Vec::new(),
            state,
            max_height: 0.0,
        };
        chart_component.update_chart(transactions);
        chart_component.max_height = chart_component.get_max_bar_height().unwrap();
        chart_component
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

    fn get_current_item(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('k') => self.next_row(),
            KeyCode::Char('l') => self.previous_row(),
            _ => (),
        }
    }

    pub fn update_chart(&mut self, transactions: &Vec<models::Transaction>) {
        self.items = get_transactions_by_month(
            &transactions
                .iter()
                .cloned()
                .filter(|row| (row.kind == Kind::DebitPurchase || row.kind == Kind::CreditPurchase))
                .collect::<Vec<models::Transaction>>(),
        );
        self.max_height = self.get_max_bar_height().unwrap();
    }

    fn get_max_bar_height(&self) -> Option<f64> {
        self.items.iter().map(|row| row.total).reduce(f64::max)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(3), Constraint::Fill(1)],
        )
        .split(area);
        self.render_chart(frame, layout[0]);
        self.render_list(frame, layout[1]);
    }

    fn render_chart(&self, frame: &mut Frame, area: Rect) {
        let current_month_expense = &self.items[self.get_current_item().unwrap()];

        let block = Block::default()
            .title("Expenses by month")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let barchart = vertical_barchart(&current_month_expense.categorized)
            .max(self.max_height as u64)
            .block(block);

        frame.render_widget(barchart, area);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Expenses list")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        let items = self
            .items
            .iter()
            .map(|item| ListItem::from(item))
            .collect::<Vec<ListItem>>();

        let list = List::new(items).block(block).highlight_symbol(">");

        StatefulWidget::render(list, area, frame.buffer_mut(), &mut self.state);
    }
}

impl From<&MonthExpense> for ListItem<'_> {
    fn from(value: &MonthExpense) -> Self {
        let line = Line::from(format!(
            "{} - {}: R${}",
            value.month, value.year, value.total
        ));
        ListItem::new(line)
    }
}
