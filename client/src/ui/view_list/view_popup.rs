// TODO: view popups
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer, layout::{Constraint, Flex, Layout, Rect}, style::{Style, Stylize}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget}
};
use tui_textarea::{TextArea, TextAreaWidget};
use thiserror::Error;

use crate::mid::{ModifyTaskError, NoTaskError, State, Task, TaskKey, ViewKey, View};

#[derive(Debug)]
pub enum ViewPopup {
	Create(String),
	Delete(ViewKey, String),
    Edit(ViewKey, Box<TextArea>),
}

#[derive(Debug, Error)]
pub enum ViewPopupCloseError {
    #[error("should handle to make sure view exists in shown views")]
    AddView(ViewKey),
}

impl ViewPopup {
//     pub fn edit(key: ViewKey, state: &State) -> Option<Self> {
//         let name = &state.view_get(key).ok()?.name;
//         let mut textarea = TextArea::default();
//         textarea.set_cursor_line_style(Style::default());
//         textarea.insert_str(name);

//         Some(Self::Edit(key, Box::new(textarea)))
//     }
    
    /// returns Ok with boolean notifying calling event handler whether to trigger re-render.
    /// returns Err with optional error if popup should be closed
    pub fn handle_term_event(&mut self, state: &mut State, event: &Event) -> Result<bool, Option<ViewPopupCloseError>> {
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
                        let mut view = View::default();
                        view.name = name.clone();
                        let view_key = state.view_def(view);
                        return Err(Some(ViewPopupCloseError::AddView(view_key)));
                    }
                    _ => return Ok(false),
                }
            },
            Self::Delete(key, _) => {
                todo!();
            },
            Self::Edit(key, textarea) => {
                todo!();
            },
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
                .title("Create View")
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