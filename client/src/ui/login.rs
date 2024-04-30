use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

#[derive(Debug)]
pub struct Login {
    email: String,
    name: String,
    error_message: String,
    pub curser_on_name: bool,
    pub should_close: bool,
}

impl Login {
    pub fn new() -> Login {
        Self {
            name: String::new(),
            email: String::new(),
            error_message: String::new(),
            curser_on_name: true,
            should_close: false,
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // centered rectangle for login form
        let vertical_center = Layout::vertical([Constraint::Length(1000)]).split(area);
        let login_area =
            Layout::horizontal([Constraint::Percentage(100)]).split(vertical_center[0])[0];
        Clear.render(login_area, buf);
        let block = Block::default()
            .title("Login")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        Paragraph::new(format!(
            "Name: {}\nEmail: {}\n{}",
            self.name, self.email, self.error_message
        ))
        .block(block)
        .render(login_area, buf);
    }

    pub fn handle_key_event(&mut self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Enter => {
                if self.email.is_empty() || self.name.is_empty() {
                    // self.error_message = Some("Email and name are required.".into());
                    self.error_message = "Email and name are required.".to_owned();
                } else {
                    self.should_close = true;
                }
            }
            KeyCode::Backspace => {
                if self.curser_on_name {
                    if !self.name.is_empty() {
                        self.name.pop();
                    }
                } else {
                    if !self.email.is_empty() {
                        self.email.pop();
                    }
                }
            }

            KeyCode::Char(c) => {
                if self.curser_on_name {
                    self.name.push(c);
                } else {
                    self.email.push(c);
                }
            }
            _ => return false,
        }
        true
    }
}
