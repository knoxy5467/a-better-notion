use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget},
};
use thiserror::Error;
use tui_textarea::{TextArea, TextAreaWidget};

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
    pub fn handle_term_event(
        &mut self,
        state: &mut State,
        event: &Event,
    ) -> Result<bool, Option<CloseError>> {
        // Esc to exit
        if let Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) = event
        {
            return Err(None);
        }
        // match on variant
        match self {
            Self::Create(name) => {
                let Event::Key(KeyEvent { code, .. }) = event else {
                    return Ok(false);
                };
                match code {
                    KeyCode::Char(c) => name.push(*c),
                    KeyCode::Backspace => {
                        name.pop();
                    }
                    KeyCode::Enter => {
                        let task_key = state.task_def(Task::new(name.clone(), false));
                        return Err(Some(CloseError::AddTask(task_key)));
                    }
                    _ => return Ok(false),
                }
            }
            Self::Delete(key, _) => {
                let Event::Key(KeyEvent { code, .. }) = event else {
                    return Ok(false);
                };
                match code {
                    KeyCode::Char('n') => return Err(None),
                    KeyCode::Char('y') => return Err(state.task_rm(*key).err().map(Into::into)),
                    _ => return Ok(false),
                }
            }
            Self::Edit(key, textarea) => {
                if let Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) = event
                {
                    return Err(state
                        .task_mod(*key, |t| {
                            if let Some(line) = textarea.lines().first() {
                                t.name.clone_from(line)
                            }
                        })
                        .err()
                        .map(Into::into));
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
            }
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

                Paragraph::new(text).block(block).render(popup_area, buf);
            }
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

#[cfg(test)]
mod task_popup_tests {
    mod render_tests {
        use chrono::format;
        use crossterm::event::{KeyCode, KeyEvent};
        use ratatui::{buffer::Buffer, layout::Rect};

        use crate::mid::TaskKey;

        use super::super::TaskPopup;

        #[test]
        fn test_create() {
            let mut task_popup: TaskPopup = TaskPopup::Create(String::from("Test Task")); // Initialize a TaskPopup object
            let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 10)); // Initialize a buffer with a certain size
            let rect = Rect::new(0, 0, 100, 10); // Initialize a rectangle with a certain size
            task_popup.render(rect, &mut buffer);
            assert!(format!("{:?}", buffer).contains("Create Task")); // Check if the buffer contains the string "Create Task"
            assert!(format!("{:?}", buffer).contains("Test Task")); // Check if the buffer contains the string "Test Task"
        }
        #[test]
        fn test_delete() {
            let mut task_popup: TaskPopup =
                TaskPopup::Delete(TaskKey::default(), String::from("Test Task")); // Initialize a TaskPopup object
            let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 10)); // Initialize a buffer with a certain size
            let rect = Rect::new(0, 0, 100, 10); // Initialize a rectangle with a certain size
            task_popup.render(rect, &mut buffer);
            assert!(format!("{:?}", buffer).contains("Delete Task")); // Check if the buffer contains the string "Delete Task"
            assert!(format!("{:?}", buffer).contains("Test Task")); // Check if the buffer contains the string "Test Task"
        }
    }
    mod term_events_tests {
        mod create_popup_tests {
            use ratatui::style::Modifier;

            use super::super::super::TaskPopup;
            use crate::mid::{State, TaskKey};

            #[tokio::test]
            async fn test_create_popup_yes() {
                let mut task_popup: TaskPopup = TaskPopup::Create(String::from("Test Task")); // Initialize a TaskPopup object
                let (mut state, _) = super::super::super::State::new();
                let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Enter,
                    crossterm::event::KeyModifiers::empty(),
                ));
                let result = task_popup.handle_term_event(&mut state, &event); // Call the handle_term_event function
                let result = task_popup.handle_term_event(&mut state, &event);
                assert_eq!(result.unwrap(), true) // Check if the result is Ok with a value of true
            }
        }
    }
}
