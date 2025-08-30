use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::app::button::Button;
use crate::app::categoryfield::CategoryField;
use crate::app::color::{PALETTES, TableColors};
use crate::app::stringfield::StringField;
use crate::models::{Category, Transaction};

#[derive(Clone, Default, PartialEq, Eq)]
pub enum PopupFocus {
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

#[derive(Clone)]
pub struct ItemStyle {
    pub selected: Style,
    pub non_selected: Style,
}

impl ItemStyle {
    pub fn new(colors: &TableColors) -> Self {
        Self {
            non_selected: Style::default().fg(colors.header_fg).bg(colors.header_bg),
            selected: Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(colors.selected_row_style_fg),
        }
    }
}

pub struct PopupForm {
    focus: PopupFocus,
    title: StringField,
    category: CategoryField,
    button: Button,
    transaction: Transaction,
    colors: TableColors,
}

impl PopupForm {
    pub fn new(transaction: Transaction) -> Self {
        let title_label = "Title : ";
        let max_len = 60 - 2 - 2 - (title_label.len() + 2);

        let colors = TableColors::new(&PALETTES[0]);
        let item_style = ItemStyle::new(&colors);

        PopupForm {
            title: StringField::new(title_label, &transaction.title, max_len, item_style.clone()),
            category: CategoryField::new(
                "Category",
                match &transaction.group {
                    Some(category) => category.clone(),
                    None => Category::Other,
                },
                item_style.clone(),
            ),
            button: Button::new("Ok", item_style.clone()),
            focus: PopupFocus::default(),
            transaction: transaction,
            colors,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Transaction> {
        match key_event.code {
            KeyCode::Tab => {
                self.next_field();
                return None;
            }
            KeyCode::Esc => return Some(self.get_original_transaction()),
            _ => (),
        };
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
        if self.title.get_value().len() < 3 {
            return false;
        }
        true
    }
    fn next_field(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn get_original_transaction(&self) -> Transaction {
        self.transaction.clone()
    }

    pub fn get_transaction(&self) -> Transaction {
        let mut transaction = self.transaction.clone();
        transaction.title = self.title.get_value();
        transaction.group = Some(self.category.value());
        transaction
    }

    pub fn render(&self, frame: &mut Frame) {
        let popup_block = Block::default()
            .title("Edit Transaction")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(self.colors.buffer_bg));

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
                Constraint::Length(2),
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
                "Use Tab to navigate between fields\n
                 and arrows to choose a Category.",
            )
            .fg(self.colors.row_fg)
            .centered()
            .wrap(Wrap { trim: true }),
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
            Constraint::Min(10),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Min(60),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
