use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Offset, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget};
use strum::IntoEnumIterator;

use crate::models::{Category, Transaction};

struct StringField {
    label: &'static str,
    value: String,
    max_length: usize,
}

impl StringField {
    pub fn new(label: &'static str, value: &String, max_length: usize) -> Self {
        Self {
            label,
            value: value.clone(),
            max_length,
        }
    }
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                if self.value.len() < self.max_length {
                    self.value.push(c);
                }
            }
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => (),
        }
    }

    fn cursor_offset(&self) -> Offset {
        let x = (self.label.len() + self.value.len() + 2) as i32;
        Offset { x: x, y: 0 }
    }
}

impl Widget for &StringField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .split(area);
        let label = Line::from_iter([self.label, ": "]).bold();
        label.render(layout[0], buf);
        self.value
            .clone()
            .bg(Color::DarkGray)
            .render(layout[1], buf);
    }
}

struct CategoryField {
    label: &'static str,
    categories: Vec<Category>,
    selected: Category,
}

impl CategoryField {
    pub fn new(label: &'static str, value: Category) -> Self {
        let categories: Vec<Category> = Category::iter().collect();
        Self {
            label,
            categories,
            selected: value,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Right | KeyCode::Char(';') => self.next(),
            KeyCode::Left | KeyCode::Char('j') => self.previous(),
            _ => (),
        }
    }

    fn next(&mut self) {
        self.selected = self.selected.next();
    }

    fn previous(&mut self) {
        self.selected = self.selected.previous();
    }

    pub fn value(&self) -> Category {
        self.selected.clone()
    }
}

impl Widget for &CategoryField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .split(area);

        let label = Line::from_iter([self.label, ": "]).bold();
        label.render(layout[0], buf);
        let value = format!("< {} >", self.value());
        value
            .clone()
            .to_string()
            .bg(Color::DarkGray)
            .render(layout[1], buf);
    }
}

enum ButtonState {
    Selected,
    NotSelected,
}

struct Button {
    label: &'static str,
    state: ButtonState,
}

impl Button {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            state: ButtonState::NotSelected,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if key_event.code == KeyCode::Enter {
            return true;
        };
        return false;
    }
}

impl StatefulWidget for &Button {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ButtonState) {
        let label = Line::from(self.label).bold();
        let background_color = match state {
            ButtonState::NotSelected => Color::Black,
            ButtonState::Selected => Color::DarkGray,
        };
        let button = Paragraph::new(label)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::new().borders(Borders::ALL))
            .bg(background_color);
        button.render(area, buf);
    }
}

#[derive(Default, PartialEq, Eq)]
enum PopupFocus {
    #[default]
    Title,
    Category,
    Ok,
}

impl PopupFocus {
    const fn next(&self) -> Self {
        match self {
            Self::Title => Self::Category,
            Self::Category => Self::Ok,
            Self::Ok => Self::Title,
        }
    }
}

pub struct PopupForm {
    focus: PopupFocus,
    title: StringField,
    category: CategoryField,
    confirm: Button,
    transaction: Transaction,
}

impl PopupForm {
    pub fn new(transaction: Transaction) -> Self {
        let title_label = "Title";
        let max_len = 40 - 2 - 2 - (title_label.len() + 2);

        PopupForm {
            title: StringField::new(title_label, &transaction.title, max_len),
            category: CategoryField::new(
                "Category",
                match &transaction.group {
                    Some(category) => category.clone(),
                    None => Category::Other,
                },
            ),
            confirm: Button::new("Ok"),
            focus: PopupFocus::default(),
            transaction: transaction,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Transaction> {
        if key_event.code == KeyCode::Tab {
            self.next_field();
            return None;
        }
        match self.focus {
            PopupFocus::Title => self.title.handle_key_event(key_event),
            PopupFocus::Category => self.category.handle_key_event(key_event),
            PopupFocus::Ok => {
                if self.confirm.handle_key_event(key_event) {
                    return Some(self.get_transaction());
                }
            }
        }
        return None;
    }

    fn next_field(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn get_transaction(&self) -> Transaction {
        let mut transaction = self.transaction.clone();
        transaction.title = self.title.value.clone();
        transaction.group = Some(self.category.value());
        transaction
    }

    pub fn render(&self, frame: &mut Frame) {
        let popup_block = Block::default()
            .title("Edit Transaction")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let area = centered_rect(40, 10, frame.area());

        frame.render_widget(Clear, area);
        frame.render_widget(popup_block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(area);

        frame.render_widget(&self.title, layout[0]);
        frame.render_widget(&self.category, layout[1]);

        let button_area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .split(layout[2])[1];

        match self.focus {
            PopupFocus::Ok => {
                frame.render_stateful_widget(&self.confirm, button_area, &mut ButtonState::Selected)
            }
            _ => frame.render_stateful_widget(
                &self.confirm,
                button_area,
                &mut ButtonState::NotSelected,
            ),
        }

        let cursor_position = match self.focus {
            PopupFocus::Title => layout[0].offset(self.title.cursor_offset()),
            PopupFocus::Category => layout[1],
            PopupFocus::Ok => button_area,
        };

        frame.set_cursor_position(cursor_position);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Min(8),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Min(40),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

