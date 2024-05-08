use std::{
    cmp::{max, min},
    ptr,
};

use color_eyre::owo_colors::OwoColorize;
use common::*;
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
    },
    Delete(ViewKey, String),
    Edit(ViewKey, Box<TextArea>),
}
#[derive(Debug, Clone)]
pub struct States {
    pub line: usize,
    pub state: CreateState,
    pub filter: Filter,
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

fn traverse_filter<'a>(
    res: &mut Vec<(String, &'a Filter, &'a Filter)>,
    filter: &'a Filter,
    parent: &'a Filter,
    depth: i32,
) {
    res.push((
        format!("{}{}", "  ".repeat(depth as usize), filter),
        filter,
        parent,
    ));
    if let Filter::Operator { op: _, childs } = filter {
        for child in childs {
            traverse_filter(res, child, filter, depth + 1);
        }
    };
}

fn delete_node_copy(
    filter: &Filter,
    parent: &Filter,
    target: &Filter,
) -> Result<Filter, &'static str> {
    if let Filter::Operator { op, childs } = filter {
        if ptr::eq(filter, parent) {
            let mut new_children = vec![];
            for child in childs {
                if !ptr::eq(child, target) {
                    new_children.push(child.clone())
                }
            }
            return Ok(Filter::Operator {
                op: op.clone(),
                childs: new_children,
            });
        }
        return Ok(Filter::Operator {
            op: op.clone(),
            childs: childs
                .iter()
                .map(|child| delete_node_copy(child, parent, target).unwrap())
                .collect(),
        });
    }

    //if it's a leaf we can't find it here
    Ok(filter.clone())
}
fn add_node_copy(
    filter: &Filter,
    parent: &Filter,
    target: &Filter,
) -> Result<Filter, &'static str> {
    if let Filter::Operator { op, childs } = filter {
        if ptr::eq(filter, parent) {
            return Ok(Filter::Operator {
                op: op.clone(),
                childs: childs
                    .iter()
                    .map(|child| add_node_copy(child, filter, target).unwrap())
                    .collect(),
            });
        }
        return Ok(Filter::Operator {
            op: op.clone(),
            childs: childs
                .iter()
                .map(|child| delete_node_copy(child, parent, target).unwrap())
                .collect(),
        });
    }

    //if it's a leaf we can't find it here
    Ok(filter.clone())
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
                    (CreateState::Filter, KeyCode::Up) => {
                        stat.line -= 1;
                    }
                    (CreateState::Filter, KeyCode::Down) => {
                        stat.line += 1;
                    }
                    (CreateState::Filter, KeyCode::Delete) => {
                        let mut res: Vec<(String, &Filter, &Filter)> = vec![];
                        traverse_filter(&mut res, &stat.filter, &Filter::None, 0);

                        // remove child from parent
                        if let Filter::Operator { op: _, childs: _ } = res[stat.line].2 {
                            stat.filter =
                                delete_node_copy(&stat.filter, res[stat.line].2, res[stat.line].1)
                                    .unwrap();
                        }
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
                CreateState::Filter => {
                    let block = Block::default()
                        .title("Create Filter")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    let mut res: Vec<(String, &Filter, &Filter)> = vec![];
                    traverse_filter(&mut res, &stat.filter, &Filter::None, 0);

                    // clamp the index please
                    stat.line = min(max(0, stat.line), res.len());

                    let text: Vec<Line> = res
                        .iter()
                        .enumerate()
                        .map(|(i, (s, _, _))| {
                            if i == stat.line {
                                Line::from(Span::styled(s, Style::new().red().bold()))
                            } else {
                                Line::from(Span::raw(s))
                            }
                        })
                        .collect();

                    Paragraph::new(text).block(block).render(area, buf);
                }
                _ => {}
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

#[cfg(test)]
mod test_stuff {
    use common::*;

    use crate::ui::view_list::view_popup::delete_node_copy;

    use super::traverse_filter;

    #[test]
    fn test_filter_stuff() {
        let filter = Filter::Operator {
            op: Operator::AND,
            childs: vec![
                Filter::Operator {
                    op: Operator::OR,
                    childs: vec![
                        Filter::Leaf {
                            field: "dogs".to_string(),
                            comparator: Comparator::EQ,
                            immediate: TaskPropVariant::Boolean(true),
                        },
                        Filter::Leaf {
                            field: "dogs".to_string(),
                            comparator: Comparator::EQ,
                            immediate: TaskPropVariant::Boolean(true),
                        },
                    ],
                },
                Filter::Leaf {
                    field: "dogs".to_string(),
                    comparator: Comparator::EQ,
                    immediate: TaskPropVariant::Boolean(true),
                },
            ],
        };
        //get vec
        let mut res: Vec<(String, &Filter, &Filter)> = vec![];
        traverse_filter(&mut res, &filter, &Filter::None, 0);
        for (s, _, _) in res.iter() {
            println!("{}", s);
        }
        println!("{}{}", res[3].2, res[3].1);
        println!("BREAK");

        //try to delete things
        let new_filter = delete_node_copy(&filter, res[3].2, res[3].1).unwrap();
        res.clear();
        traverse_filter(&mut res, &new_filter, &Filter::None, 0);
        for (s, _, _) in res.iter() {
            println!("{}", s);
        }
    }
}
