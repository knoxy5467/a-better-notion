use color_eyre::owo_colors::OwoColorize;
use common::Filter;
// TODO: view popups
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

use crate::mid::{ModifyTaskError, NoTaskError, State, Task, TaskKey, View, ViewKey};

#[derive(Debug)]
pub enum ViewPopup {
    Create {
        edit: String,
        edit_leaf: Filter,
        stat: States,
        err: String,
        name: String,
        props: Vec<String>,
        fitler: Filter,
    },
    Delete(ViewKey, String),
    Edit(ViewKey, Box<TextArea>),
}
#[derive(Debug, Clone)]
pub struct States {
    pub line: i32,
    pub state: CreateState,
}

#[derive(Debug, Clone)]
pub enum CreateState {
    Name,
    Props,
    Filter,
    FilterNameType,
    FilterPropName,
    FilterPropImm,
    FilterConditional,
    FilterAndOrNot,
}

#[derive(Debug, Error)]
pub enum ViewPopupCloseError {
    #[error("should handle to make sure view exists in shown views")]
    AddView(ViewKey),
}

fn traverse_filter(res: &mut Vec<(i32, String, Filter)>, filter: Filter, depth: i32) {
    match filter {
        Filter::Leaf {
            field: _,
            comparator: _,
            immediate: _,
        } => res.push((depth, format!("{}", filter.clone()), filter)),
        Filter::Operator { op, childs } => {
            res.push((depth, format!("{}", filter.clone()), filter));
            for child in childs {
                traverse_filter(res, child, depth + 1);
            }
        }
        Filter::None => todo!(),
    }
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
    pub fn handle_term_event(
        &mut self,
        state: &mut State,
        event: &Event,
    ) -> Result<bool, Option<ViewPopupCloseError>> {
        // Esc to exit
        if let Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) = event
        {
            return Err(None);
        }
        // match on variant
        match self {
            Self::Create {
                edit,
                edit_leaf,
                stat,
                err,
                name,
                props,
                fitler,
            } => {
                let Event::Key(KeyEvent { code, .. }) = event else {
                    return Ok(false);
                };
                match (stat.state.clone(), code) {
                    (CreateState::Name | CreateState::Props, KeyCode::Char(c)) => edit.push(*c),
                    (CreateState::Name | CreateState::Props, KeyCode::Backspace) => {
                        edit.pop();
                    }
                    (CreateState::Name, KeyCode::Enter) => {
                        name.replace_range(.., edit);
                        edit.clear();
                        stat.state = CreateState::Props
                    }
                    (CreateState::Props, KeyCode::Enter) => {
                        if edit.is_empty() {
                            stat.state = CreateState::Filter
                        }
                        props.push(edit.clone());
                        edit.clear();
                    }
                    _ => (),
                }
            }
            Self::Delete(key, _) => {
                todo!();
            }
            Self::Edit(key, textarea) => {
                todo!();
            }
        }
        Ok(true)
    }
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // create a centered rect of fixed vertical size that takes up 50% of the vertical area.
        let vertical_center = Layout::vertical([Constraint::Length(5)])
            .flex(Flex::Center)
            .split(area);

        let popup_area = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .split(vertical_center[0])[0];

        // Clear.render(popup_area, buf); // clear background of popup area
        Clear.render(area, buf); // clear background of popup area

        match self {
            Self::Create {
                edit,
                edit_leaf,
                stat,
                err,
                name,
                props,
                fitler,
            } => match stat.state.clone() {
                CreateState::Name => {
                    let block = Block::default()
                        .title("Create View")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    Paragraph::new(edit.as_str()).block(block).render(area, buf);
                }
                CreateState::Props => {
                    let block = Block::default()
                        .title("List Properties")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    let mut text: Vec<Line> = props
                        .iter()
                        .map(|prop| Line::from(Span::raw(prop)))
                        .collect();
                    text.push(Line::from(Span::raw(edit.clone())));
                    Paragraph::new(text).block(block).render(area, buf)
                }
                CreateState::Filter => {}
                _ => {
                    let block = Block::default();
                    Paragraph::new("heyo").block(block).render(popup_area, buf);
                }
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
