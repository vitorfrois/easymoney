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

impl StatefulWidget for &StringField {
    type State = PopupFocus;
    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut PopupFocus) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .split(area);
        let label = Line::from_iter([self.label, ": "]).bold();
        label.render(layout[0], buf);
        let bg = Color::Black;
        self.value.clone().bg(bg).render(layout[1], buf);
    }
}

struct CategoryField {
    label: &'static str,
    selected: Category,
}

impl CategoryField {
    pub fn new(label: &'static str, value: Category) -> Self {
        Self {
            label,
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

impl StatefulWidget for &CategoryField {
    type State = PopupFocus;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PopupFocus) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .split(area);

        let label = Line::from_iter([self.label, ": "]).bold();
        label.render(layout[0], buf);
        let value = format!("< {} >", self.value());
        let bg = match state {
            PopupFocus::Category => Color::DarkGray,
            _ => Color::Black,
        };
        value.clone().to_string().bg(bg).render(layout[1], buf);
    }
}

struct Button {
    label: &'static str,
}

impl Button {
    pub fn new(label: &'static str) -> Self {
        Self { label }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if key_event.code == KeyCode::Enter {
            return true;
        };
        return false;
    }
}

impl StatefulWidget for &Button {
    type State = PopupFocus;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PopupFocus) {
        let label = Line::from(self.label).bold();
        let bg = match state {
            PopupFocus::Ok => Color::DarkGray,
            _ => Color::Black,
        };
        let button = Paragraph::new(label)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::new().borders(Borders::ALL))
            .bg(bg);
        button.render(area, buf);
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
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
    button: Button,
    transaction: Transaction,
}

impl PopupForm {
    pub fn new(transaction: Transaction) -> Self {
        let title_label = "Title";
        let max_len = 60 - 2 - 2 - (title_label.len() + 2);

        PopupForm {
            title: StringField::new(title_label, &transaction.title, max_len),
            category: CategoryField::new(
                "Category",
                match &transaction.group {
                    Some(category) => category.clone(),
                    None => Category::Other,
                },
            ),
            button: Button::new("Ok"),
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
                if self.button.handle_key_event(key_event) {
                    if self.validate_title() {
                        return Some(self.get_transaction());
                    } else {
                        return None;
                    }
                }
            }
        }
        return None;
    }

    fn validate_title(&self) -> bool {
        if self.title.value.len() < 3 {
            return false;
        }
        true
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
                Constraint::Length(1),
            ])
            .split(area);

        let button_area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .split(layout[2])[1];

        frame.render_stateful_widget(&self.title, layout[0], &mut self.focus.clone());
        frame.render_stateful_widget(&self.category, layout[1], &mut self.focus.clone());
        frame.render_stateful_widget(&self.button, button_area, &mut self.focus.clone());

        let message = match !self.validate_title() {
            true => Paragraph::new("Use 3 or more chars")
                .fg(Color::Red)
                .centered(),
            false => Paragraph::new(
                "Use Tab to navigate between fields. Use arrows to choose a Category.",
            )
            .fg(Color::White)
            .centered(),
        };
        frame.render_widget(message, layout[3]);

        match self.focus {
            PopupFocus::Title => {
                let cursor_position = layout[0].offset(self.title.cursor_offset());
                frame.set_cursor_position(cursor_position);
            }
            _ => (),
        };
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
