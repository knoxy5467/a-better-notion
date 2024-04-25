use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer, layout::{Constraint, Flex, Layout, Rect}, style::{Style, Stylize}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget}
};
use tui_textarea::{TextArea, TextAreaWidget};
use thiserror::Error;

use crate::mid::{ModifyTaskError, NoTaskError, State, Task, TaskKey};

#[derive(Debug)]
pub enum TaskPopup {
	Create(String),
	Delete(TaskKey, String),
    Edit(TaskKey, Box<TextArea>),
}

#[derive(Debug, Error)]
pub enum CloseError {
    #[error(transparent)]
    NoTaskError(#[from] NoTaskError),
    #[error(transparent)]
    ModifyTaskError(#[from] ModifyTaskError),
    #[error("should handle to make sure task exists in shown tasks")]
    AddTask(TaskKey),
}

impl TaskPopup {
    pub fn edit(key: TaskKey, state: &State) -> Option<Self> {
        let name = &state.task_get(key).ok()?.name;
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.insert_str(name);

        Some(Self::Edit(key, Box::new(textarea)))
    }
    
    /// returns Ok with boolean notifying calling event handler whether to trigger re-render.
    /// returns Err with optional error if popup should be closed
    pub fn handle_term_event(&mut self, state: &mut State, event: &Event) -> Result<bool, Option<CloseError>> {
        // Esc to exit
        if let Event::Key(KeyEvent{code: KeyCode::Esc,..}) = event { return Err(None) }
        // match on variant
        match self {
            Self::Create(name) => {
                let Event::Key(KeyEvent { code, .. }) = event else {return Ok(false)};
                match code {
                    KeyCode::Char(c) => name.push(*c),
                    KeyCode::Backspace => {name.pop();},
                    KeyCode::Enter => {
                        let task_key = state.task_def(Task::new(name.clone(), false));
                        return Err(Some(CloseError::AddTask(task_key)));
                    }
                    _ => return Ok(false),
                }
            },
            Self::Delete(key, _) => {
                let Event::Key(KeyEvent { code, .. }) = event else {return Ok(false)};
                match code {
                    KeyCode::Char('n') => return Err(None),
                    KeyCode::Char('y') => {
                        return Err(state.task_rm(*key).err().map(Into::into))
                    }
                    _ => return Ok(false),
                }
            },
            Self::Edit(key, textarea) => {
                if let Event::Key(KeyEvent{code: KeyCode::Enter, ..}) = event {
                    return Err(state.task_mod(*key, |t|
                        if let Some(line) = textarea.lines().first() {t.name.clone_from(line)}
                    ).err().map(Into::into));
                } else {
                    textarea.input(event.clone());
                }
            }
        }
        Ok(true)
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
		
		match self {
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
                        Span::styled(&*name, Style::new().italic()),
                        Span::raw("\""),
                    ]),
                    Line::from("Delete: [Y/N]"),
                ];

                Paragraph::new(text)
                .block(block)
                .render(popup_area, buf);
            },
            Self::Edit(_key, textarea) => {
                let block = Block::default()
                    .title("Edit Task")
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED);
                let widget = TextAreaWidget::new().block(block);
                StatefulWidget::render(widget, popup_area, buf, textarea)
            }
		};
    }
}

/* #[cfg(test)]
mod tests {
    async fn mock_popup() {

    }
} */
