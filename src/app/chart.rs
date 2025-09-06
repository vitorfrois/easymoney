use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, BorderType, Borders, Cell, Chart, Dataset, List,
        ListItem, ListState, Row, StatefulWidget, Table, TableState, Widget,
    },
};
use strum::{Display, IntoEnumIterator};

use crate::models::{self, Category, Kind, Transaction};
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    iter::zip,
};

const NULL_KEY: &str = "[N/A]";
const NEEDS: [Category; 5] = [
    Category::Food,
    Category::Housing,
    Category::Transportation,
    Category::Supermarket,
    Category::Health,
];
const SAVINGS: [Category; 1] = [Category::Savings];
const WANTS: [Category; 3] = [Category::Trips, Category::Personal, Category::Other];

fn vertical_barchart(
    category_totals: &HashMap<String, f64>,
    bar_width: u16,
    bar_gap: u16,
    max_size: u64,
    percentage: bool,
) -> BarChart {
    let bars: Vec<Bar> = category_totals
        .iter()
        .sorted_by(|a, b| a.0.cmp(&b.0))
        .map(|(category, value)| {
            let text = match percentage {
                true => format!("{:.2}%", value),
                false => format!("{:.2}", value),
            };
            vertical_bar(category.clone(), value, text)
        })
        .collect();

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .max(max_size)
}

fn vertical_bar(category: String, value: &f64, text_value: String) -> Bar {
    Bar::default()
        .value(*value as u64)
        .label(Line::from(category))
        .text_value(text_value)
}

fn get_transactions_by_category(transactions: &Vec<models::Transaction>) -> HashMap<String, f64> {
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

fn get_transactions_by_month(transactions: &Vec<models::Transaction>) -> Vec<MonthSummary> {
    let month_set: HashSet<(i32, u32)> = HashSet::from(
        transactions
            .iter()
            .map(|row| (row.date.year(), row.date.month()))
            .collect::<HashSet<(i32, u32)>>(),
    );

    let month_expenses = month_set
        .iter()
        .map(|(year, month)| MonthSummary::new(transactions, *year, *month))
        .sorted_by(|a, b| (a.month as i32 + a.year).cmp(&(b.month as i32 + b.year)))
        .rev()
        .collect();

    month_expenses
}

#[derive(PartialEq, Debug)]
struct FiftyThirtyTwenty {
    needs: f64,
    wants: f64,
    savings: f64,
}

impl FiftyThirtyTwenty {
    fn reduce_by(transactions: &Vec<Transaction>, list: &[Category]) -> f64 {
        transactions
            .iter()
            .filter(|row| match &row.group {
                Some(category) => list.contains(&category),
                None => false,
            })
            .map(|row| row.amount)
            .sum()
    }

    fn new(transactions: &Vec<Transaction>) -> Self {
        let needs = FiftyThirtyTwenty::reduce_by(transactions, &NEEDS);
        let wants = FiftyThirtyTwenty::reduce_by(transactions, &WANTS);
        let savings = FiftyThirtyTwenty::reduce_by(transactions, &SAVINGS);
        Self {
            needs,
            wants,
            savings,
        }
    }
}

#[derive(PartialEq, Debug)]
struct MonthSummary {
    year: i32,
    month: u32,
    total_income: f64,
    total_expenses: f64,
    fifty_thirty_twenty: FiftyThirtyTwenty,
    categorized_expenses: HashMap<String, f64>,
}

impl MonthSummary {
    fn ref_array(&self) -> [Cow<str>; 3] {
        let expenses_string = format!("{:.2}", self.total_expenses);
        let income_string = format!("{:.2}", self.total_income);
        let date = format!("{:2}/{}", self.month, self.year);
        [
            Cow::Owned(date),
            Cow::Owned(expenses_string),
            Cow::Owned(income_string),
        ]
    }
}

impl MonthSummary {
    fn new(transactions: &Vec<Transaction>, year: i32, month: u32) -> Self {
        let month_transactions: Vec<Transaction> = transactions
            .clone()
            .into_iter()
            .filter(|row| row.date.month() == month && row.date.year() == year)
            .collect();
        let expenses: Vec<Transaction> = month_transactions
            .clone()
            .into_iter()
            .filter(|row| (row.kind == Kind::DebitPurchase || row.kind == Kind::CreditPurchase))
            .collect();
        let total_income = month_transactions
            .iter()
            .filter(|row| (row.kind == Kind::Income))
            .map(|row| row.amount)
            .sum();

        let total_expenses = expenses.iter().map(|row| row.amount).sum();
        let fifty_thirty_twenty = FiftyThirtyTwenty::new(&expenses);
        let categorized_expenses = get_transactions_by_category(&expenses);
        Self {
            year,
            month,
            total_income,
            total_expenses,
            fifty_thirty_twenty,
            categorized_expenses,
        }
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub struct ChartComponent {
    items: Vec<MonthSummary>,
    state: TableState,
    max_height: f64,
}

impl ChartComponent {
    pub fn new(transactions: &Vec<models::Transaction>) -> Self {
        let mut state = TableState::default();
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
                .clone()
                .into_iter()
                .collect::<Vec<models::Transaction>>(),
        );
        self.max_height = self.get_max_bar_height().unwrap();
    }

    fn get_max_bar_height(&self) -> Option<f64> {
        self.items
            .iter()
            .map(|row| row.total_expenses)
            .reduce(f64::max)
    }

    fn get_terminal_width() -> termsize::Size {
        termsize::get().unwrap()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Min(32), Constraint::Fill(4)],
        )
        .split(area);
        let right_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Fill(1)],
        )
        .split(layout[1]);

        let sub_right_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(3), Constraint::Fill(1)],
        )
        .split(right_layout[1]);

        self.render_rule_chart(frame, sub_right_layout[1]);
        self.render_scatter(frame, sub_right_layout[0]);
        self.render_category_chart(frame, right_layout[0]);
        self.render_list(frame, layout[0]);
    }

    fn render_category_chart(&self, frame: &mut Frame, area: Rect) {
        let current_month = &self.items[self.get_current_item().unwrap()];

        let block = Block::default()
            .title("Month by category")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = block.inner(area).inner(Margin::new(5, 0));
        frame.render_widget(block, area);

        let number_bars = (Category::iter().len() + 1) as f64;
        let bar_gap = 2.0;
        let calculated_width = (inner_area.width as f64 - (number_bars - 1.0) * bar_gap)
            / (number_bars).floor().max(1.0);
        let bar_width = calculated_width;

        let barchart = vertical_barchart(
            &current_month.categorized_expenses,
            bar_width as u16,
            bar_gap as u16,
            self.max_height as u64,
            false,
        );

        frame.render_widget(barchart, inner_area);
    }

    fn render_rule_chart(&self, frame: &mut Frame, area: Rect) {
        let current_month = &self.items[self.get_current_item().unwrap()];

        let block = Block::default()
            .title("Month by 50/30/20 Rule")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let rule_expenses: HashMap<String, f64> = zip(
            [
                "Needs".to_string(),
                "Wants".to_string(),
                "Savings".to_string(),
            ]
            .into_iter(),
            [
                current_month.fifty_thirty_twenty.needs / current_month.total_income * 100.0,
                current_month.fifty_thirty_twenty.wants / current_month.total_income * 100.0,
                current_month.fifty_thirty_twenty.savings / current_month.total_income * 100.0,
            ]
            .into_iter(),
        )
        .collect::<HashMap<String, f64>>();

        let inner_area = block.inner(area).inner(Margin::new(2, 0));
        frame.render_widget(block, area);

        let number_bars = rule_expenses.len() as f64;
        let bar_gap = 2.0;
        let calculated_width = (inner_area.width as f64 - (number_bars - 1.0) * bar_gap)
            / (number_bars).floor().max(1.0);
        let bar_width = calculated_width;

        let barchart =
            vertical_barchart(&rule_expenses, bar_width as u16, bar_gap as u16, 100, true);

        frame.render_widget(barchart, inner_area);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Expenses list")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let rows = self.items.iter().map(|item| {
            item.ref_array()
                .into_iter()
                .map(Cell::from)
                .collect::<Row>()
        });

        let header = ["Month", "Expenses", "Income"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .bold();

        let list = Table::new(
            rows,
            [Constraint::Fill(1), Constraint::Min(8), Constraint::Min(8)],
        )
        .block(block)
        .highlight_symbol(">")
        .header(header);

        StatefulWidget::render(list, area, frame.buffer_mut(), &mut self.state);
    }

    fn render_scatter(&mut self, frame: &mut Frame, area: Rect) {
        let expense_data = self
            .items
            .iter()
            .take(6)
            .map(|item| {
                (
                    convert_date_float(item.month, item.year),
                    item.total_expenses as f64,
                )
            })
            .collect::<Vec<(f64, f64)>>();

        let expense_dataset = Dataset::default()
            .name("Monthly ")
            .marker(ratatui::symbols::Marker::Dot)
            .graph_type(ratatui::widgets::GraphType::Line)
            .data(&expense_data);

        let data = vec![expense_dataset];

        let block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let minx = expense_data.iter().map(|a| a.0).reduce(f64::min).unwrap();
        let maxx = expense_data.iter().map(|a| a.0).reduce(f64::max).unwrap();
        let miny = expense_data.iter().map(|a| a.1).reduce(f64::min).unwrap();
        let maxy = expense_data.iter().map(|a| a.1).reduce(f64::max).unwrap();

        let y_axis = Axis::default().bounds([miny, maxy]).labels([
            miny.to_string(),
            ((miny + maxy) / 2.0).to_string(),
            maxy.to_string(),
        ]);

        let x_axis = Axis::default().bounds([minx, maxx]).labels([
            convert_float_string(minx),
            convert_float_string((minx + maxx) / 2.0),
            convert_float_string(maxx),
        ]);

        let chart = Chart::new(data).block(block).y_axis(y_axis).x_axis(x_axis);

        Widget::render(chart, area, frame.buffer_mut());
    }
}

fn convert_date_float(month: u32, year: i32) -> f64 {
    (year as f64 + (month as f64) / 12.0) as f64
}

fn convert_float_string(float: f64) -> String {
    let month = (float.fract() * 12.0).round() as u32;
    let year = float.trunc() as i32;
    format!("{}, {}", month, year)
}

fn convert_float_date(float: f64) -> (u32, i32) {
    let month = (float.fract() * 12.0).round() as u32;
    let year = float.trunc() as i32;
    (month, year)
}
