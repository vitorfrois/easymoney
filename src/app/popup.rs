use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Offset, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Widget};

use crate::models::{Category, Transaction};

struct StringField {
    label: &'static str,
    value: String,
}

impl StringField {
    fn new(label: &'static str, value: &String) -> Self {
        Self {
            label,
            value: value.clone(),
        }
    }
    fn on_key_press(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(c) => self.value.push(c),
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
        self.value.clone().render(layout[1], buf);
    }
}

struct CategoryField {
    label: &'static str,
    value: Category,
}

impl CategoryField {
    fn new(label: &'static str, value: Category) -> Self {
        Self { label, value }
    }
    fn on_key_press(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(';') | KeyCode::Right => self.value = self.value.next(),
            _ => (),
        }
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
        self.value.clone().to_string().render(layout[1], buf);
    }
}

#[derive(Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Title,
    Category,
}

impl Focus {
    const fn next(&self) -> Self {
        match self {
            Self::Title => Self::Category,
            Self::Category => Self::Title,
        }
    }
}

pub struct PopupForm {
    focus: Focus,
    title: StringField,
    category: CategoryField,
    transaction: Transaction,
}

impl PopupForm {
    pub fn new(transaction: &Transaction) -> Self {
        PopupForm {
            title: StringField::new("Title", &transaction.title),
            category: CategoryField::new(
                "Category",
                match &transaction.group {
                    Some(category) => category.clone(),
                    None => Category::Other,
                },
            ),
            focus: Focus::default(),
            transaction: transaction.clone(),
        }
    }

    pub fn run(&mut self, terminal: &DefaultTerminal) -> Option<Transaction> {
        loop {
            terminal
                .draw(|frame| self.render(frame))
                .expect("Error rendering");

            if let Event::Key(key) = event::read().expect("Error reading keyboard") {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return None,
                        KeyCode::Enter => {
                            return Some(self.get_transaction());
                        }
                        keycode => self.on_key_press(&keycode),
                    }
                }
            }
        }
    }

    fn on_key_press(&mut self, keycode: &KeyCode) {
        match keycode {
            KeyCode::Tab => self.focus = self.focus.next(),
            _ => (),
        }
    }

    pub fn get_transaction(&self) -> Transaction {
        let mut transaction = self.transaction.clone();
        transaction.title = self.title.value.clone();
        transaction.group = Some(self.category.value.clone());
        transaction
    }

    pub fn render(&self, frame: &mut Frame) {
        let popup_block = Block::default()
            .title("Edit Transaction")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let area = centered_rect(40, 10, frame.area());

        frame.render_widget(popup_block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        frame.render_widget(&self.title, layout[0]);
        frame.render_widget(&self.category, layout[1]);

        let cursor_position = match self.focus {
            Focus::Title => layout[0].offset(self.title.cursor_offset()),
            Focus::Category => layout[1],
        };
        frame.set_cursor_position(cursor_position);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
