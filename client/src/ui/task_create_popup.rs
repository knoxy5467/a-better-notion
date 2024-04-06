use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::mid::{State, Task};

#[derive(Debug)]
pub struct TaskCreatePopup {
    name: String,
    pub should_close: bool,
}
impl TaskCreatePopup {
    pub fn new() -> TaskCreatePopup {
        Self {
            name: Default::default(),
            should_close: false,
        }
    }
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // create a centered rect of fixed vertical size that takes up 50% of the vertical area.
        let vertical_center = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .split(area);

        let popup_area = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .split(vertical_center[0])[0];

        Clear.render(popup_area, buf); // clear background of popup area

        // create task popup block with rounded corners
        let block = Block::default()
            .title("Create Task")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        // create paragraph containing current string state inside `block` & render
        Paragraph::new(self.name.as_str())
            .block(block)
            .render(popup_area, buf);
    }
    pub fn handle_key_event(&mut self, state: &mut State, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Esc => self.should_close = true,
            KeyCode::Char(c) => {
                self.name.push(c);
            }
            KeyCode::Backspace => {
                self.name.pop();
            }
            KeyCode::Enter => {
                let task_key = state.task_def(Task {
                    name: self.name.clone(),
                    ..Default::default()
                });
                state.view_mod(state.view_get_default().unwrap(), |v| {
                    v.tasks.as_mut().unwrap().push(task_key)
                });
                self.should_close = true;
            }
            _ => return false,
        }
        true
    }
}

/* #[cfg(test)]
mod tests {
    async fn mock_popup() {

    }
} */
