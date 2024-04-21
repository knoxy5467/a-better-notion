use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer, layout::{Constraint, Flex, Layout, Rect}, style::{Style, Stylize}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, Clear, Paragraph, Widget}
};
use thiserror::Error;

use crate::mid::{NoTaskError, NoViewError, State, Task, TaskKey};

#[derive(Debug)]
pub enum TaskPopup {
	Create(String),
	Delete(TaskKey, String),
}

#[derive(Debug, Error)]
pub enum CloseError {
    #[error(transparent)]
    NoTask(#[from] NoTaskError),
    #[error(transparent)]
    NoView(#[from] NoViewError),
}

impl TaskPopup {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // create a centered rect of fixed vertical size that takes up 50% of the vertical area.
        let vertical_center = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .split(area);

        let popup_area = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .split(vertical_center[0])[0];

        Clear.render(popup_area, buf); // clear background of popup area
		
		match &self {
			Self::Create(name) => {
                let block = Block::default()
                .title("Create Task")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED);

                Paragraph::new(name.as_str())
                .block(block)
                .render(popup_area, buf);
            },
			Self::Delete(_, name) => {
                let block = Block::default()
                .title("Delete Task")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED);

                let text = vec![
                    Line::from(vec![
                        Span::styled("Deleting", Style::new().red().bold()),
                        Span::raw(" Task: \""),
                        Span::styled(name, Style::new().italic()),
                        Span::raw("\""),
                    ]),
                    Line::from("Delete: [Y/N]"),
                ];

                Paragraph::new(text)
                .block(block)
                .render(popup_area, buf);
            },
		};

        // create paragraph containing current string state inside `block` & render
        
    }
    /// returns Ok with boolean notifying calling event handler whether to trigger re-render.
    /// returns Err with optional error if popup should be closed
    pub fn handle_key_event(&mut self, state: &mut State, key_code: KeyCode) -> Result<bool, Option<NoTaskError>> {
        if let KeyCode::Esc = key_code { return Err(None) }
        match self {
            Self::Create(name) => match key_code {
                KeyCode::Char(c) => name.push(c),
                KeyCode::Backspace => {name.pop();},
                KeyCode::Enter => {
                    let task_key = state.task_def(Task::new(name.clone(), false));
                    state.view_mod(state.view_get_default().unwrap(), |v| {
                        v.tasks.as_mut().unwrap().push(task_key)
                    });
                    return Err(None);
                }
                _ => return Ok(false),
            },
            Self::Delete(key, _) => match key_code {
                KeyCode::Char('n') => return Err(None),
                KeyCode::Char('y') => {
                    return Err(state.task_rm(*key).err())
                }
                _ => return Ok(false),
            },
        }
        Ok(true)
    }
}

/* #[cfg(test)]
mod tests {
    async fn mock_popup() {

    }
} */
