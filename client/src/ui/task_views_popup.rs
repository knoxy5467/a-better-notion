use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::mid::{State, TaskKey};

#[derive(Debug)]
pub struct TaskViewsPopup {
    selected_task: TaskKey,
    pub should_close: bool,
}
impl TaskViewsPopup {
    pub fn new(task_key: TaskKey) -> TaskViewsPopup {
        Self {
            selected_task: task_key,
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

        // create task popup block

        let list = List::new(items)
            .title("Views Manager")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        // create paragraph containing current string state inside `block` & render
        Paragraph::new("You sure man? [Y/N]")
            .block(block)
            .render(popup_area, buf);
    }
    pub fn handle_key_event(&mut self, state: &mut State, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Esc       => self.should_close = true,
            KeyCode::Char('n') => self.should_close = true,
            KeyCode::Char('y') => {
                
                state.task_rm(self.selected_task);

                self.should_close = true;
            }
            _ => return false,
        }
        true
    }
}

