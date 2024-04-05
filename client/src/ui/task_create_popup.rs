use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::mid::{State, Task};

pub struct TaskCreatePopup {
    name: String,
    pub should_close: bool,
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
impl TaskCreatePopup {
    pub fn new() -> TaskCreatePopup {
        Self {
            name: Default::default(),
            should_close: false,
        }
    }
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Create Task")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);
        let area = centered_rect(60, 20, area);
        let input = Paragraph::new(self.name.as_str()).block(block);
        Clear.render(area, buf);
        input.render(area, buf);
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
