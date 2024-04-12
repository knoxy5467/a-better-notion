use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::mid::{State, TaskKey};

#[derive(Debug)]
pub struct TaskEditPopup {
    name: String,
    pub should_close: bool,
    selection: Option<TaskKey>,
    editing_mode: bool,
}

impl TaskEditPopup {
    pub fn new(selection: Option<TaskKey>) -> TaskEditPopup {
        Self {
            name: Default::default(),
            should_close: false,
            selection,
            editing_mode: false,
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // Create a centered rect of fixed vertical size that takes up 50% of the vertical area.
        let vertical_center = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .split(area);

        let popup_area = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .split(vertical_center[0])[0];

        Clear.render(popup_area, buf); // Clear background of popup area

        // Create task popup block with rounded corners
        let block = Block::default()
            .title("Editing Task")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        if self.editing_mode {
            // Render an input field for editing the task name
            Paragraph::new(self.name.as_str())
                .block(block)
                .render(popup_area, buf);
        } else {
            // Render the initial prompt
            Paragraph::new("Edit this task? [y/n]")
                .block(block)
                .render(popup_area, buf);
        }
    }

    pub fn handle_key_event(&mut self, state: &mut State, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Esc => self.should_close = true,
            KeyCode::Char('n') => self.should_close = true,
            KeyCode::Char('y') => {
                self.editing_mode = true;
            }

            KeyCode::Char(c) => {
                if self.editing_mode {
                    self.name.push(c);
                }
            }
            KeyCode::Backspace => {
                if self.editing_mode {
                    self.name.pop();
                }
            }

            KeyCode::Enter => {
                if self.editing_mode {
                    if let Some(selection) = self.selection {
                        state.task_mod(selection, |task| {
                            task.name = self.name.clone();
                        });
                        self.should_close = true;
                    }
                }
            }
            _ => return false,
        }
        true
    }
}
