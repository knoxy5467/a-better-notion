use std::{
    cmp::{max, min},
    ptr,
};

use chrono::{NaiveDate, NaiveDateTime};
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
    pub is_editing: bool,
    pub line_to_add_to: usize,
    pub edit_leaf: Filter,
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
        format!("{}{}", "  ".repeat(max(depth as usize, 1) - 1), filter),
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
                childs: childs.iter().chain([target]).cloned().collect(),
            });
        }
        return Ok(Filter::Operator {
            op: op.clone(),
            childs: childs
                .iter()
                .map(|child| add_node_copy(child, parent, target).unwrap())
                .collect(),
        });
    }

    //if it's a leaf we can't find it here
    Ok(filter.clone())
}

fn map_variant_to_allowed(filter: Filter) -> Vec<Comparator> {
    match filter {
        Filter::Leaf {
            field: _,
            comparator: _,
            immediate,
        } => match immediate {
            TaskPropVariant::Date(_) => vec![
                Comparator::LT,
                Comparator::LEQ,
                Comparator::GT,
                Comparator::GEQ,
                Comparator::EQ,
                Comparator::NEQ,
            ],
            TaskPropVariant::String(_) => vec![
                Comparator::LT,
                Comparator::LEQ,
                Comparator::GT,
                Comparator::GEQ,
                Comparator::EQ,
                Comparator::NEQ,
                Comparator::CONTAINS,
                Comparator::NOTCONTAINS,
                Comparator::REGEX,
            ],
            TaskPropVariant::Number(_) => vec![
                Comparator::LT,
                Comparator::LEQ,
                Comparator::GT,
                Comparator::GEQ,
                Comparator::EQ,
                Comparator::NEQ,
            ],
            TaskPropVariant::Boolean(_) => vec![Comparator::EQ, Comparator::NEQ],
        },
        _ => unreachable!(),
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
        // if let Event::Key(KeyEvent {
        // code: KeyCode::Esc, ..
        // }) = event
        // {
        // return Err(None);
        // }
        // match on variant
        match self {
            Self::Create {
                edit,
                stat,
                err,
                name,
                props,
            } => {
                let Event::Key(KeyEvent { code, .. }) = event else {
                    return Ok(false);
                };
                match (stat.state.clone(), code) {
                    (
                        CreateState::Name | CreateState::Props | CreateState::FilterPropImm,
                        KeyCode::Char(c),
                    ) => edit.push(*c),
                    (
                        CreateState::Name | CreateState::Props | CreateState::FilterPropImm,
                        KeyCode::Backspace,
                    ) => {
                        edit.pop();
                    }
                    (CreateState::Name, KeyCode::Enter) => {
                        if edit.is_empty() {
                            err.replace_range(.., "name should not be empty");
                            return Ok(true);
                        }

                        name.replace_range(.., edit);
                        edit.clear();
                        err.clear();
                        stat.state = CreateState::Props
                    }
                    (CreateState::Props, KeyCode::Enter) => {
                        if edit.is_empty() {
                            stat.state = CreateState::Filter
                        }
                        props.push(edit.clone());
                        edit.clear();
                    }
                    (CreateState::Filter | CreateState::FilterConditional, KeyCode::Up) => {
                        if stat.line > 0 {
                            stat.line -= 1;
                        }
                    }
                    (CreateState::Filter | CreateState::FilterConditional, KeyCode::Down) => {
                        stat.line += 1;
                    }
                    (CreateState::Filter, KeyCode::Delete) => {
                        let mut res: Vec<(String, &Filter, &Filter)> = vec![];
                        traverse_filter(&mut res, &stat.filter, &Filter::None, 0);
                        res.remove(0);

                        // can't delete plus
                        if stat.line != res.len() {
                            // remove child from parent
                            if let Filter::Operator { op: _, childs: _ } = res[stat.line].2 {
                                stat.filter = delete_node_copy(
                                    &stat.filter,
                                    res[stat.line].2,
                                    res[stat.line].1,
                                )
                                .unwrap();
                            }
                        }
                    }
                    (CreateState::Filter, KeyCode::Enter) => {
                        stat.line_to_add_to = stat.line;
                        stat.state = CreateState::FilterNameType;
                        stat.is_editing = false;
                        stat.line = 0;
                        edit.clear();
                    }
                    (CreateState::FilterNameType, KeyCode::Up) => {
                        if !stat.is_editing && (stat.line > 0) {
                            stat.line -= 1
                        }
                    }
                    (CreateState::FilterNameType, KeyCode::Down) => {
                        if !stat.is_editing {
                            stat.line += 1
                        }
                    }
                    (CreateState::FilterNameType, KeyCode::Char(c)) => {
                        if stat.is_editing {
                            edit.push(*c)
                        }
                    }
                    (CreateState::FilterNameType, KeyCode::Backspace) => {
                        if stat.is_editing {
                            edit.pop();
                        }
                    }
                    (CreateState::FilterNameType, KeyCode::Enter) => {
                        //AND, OR, title, completed, other
                        match stat.line {
                            0 | 1 => {
                                //generate inorder traversal
                                let mut res: Vec<(String, &Filter, &Filter)> = vec![];
                                traverse_filter(&mut res, &stat.filter, &Filter::None, 0);
                                let bigroot = res.remove(0);

                                let typ = if stat.line == 0 {
                                    Operator::AND
                                } else {
                                    Operator::OR
                                };

                                //add at position idk if this should be 1 or 2 :)
                                let parent = if stat.line_to_add_to == res.len() {
                                    bigroot.1 // add to BIGROOT FRICK YEAH
                                } else if let Filter::Leaf {
                                    field: _,
                                    comparator: _,
                                    immediate: _,
                                } = res[stat.line_to_add_to].1
                                {
                                    res[stat.line_to_add_to].2
                                } else {
                                    res[stat.line_to_add_to].1
                                };
                                stat.filter = add_node_copy(
                                    &stat.filter,
                                    parent,
                                    &Filter::Operator {
                                        op: typ,
                                        childs: vec![],
                                    },
                                )
                                .unwrap();
                                stat.state = CreateState::Filter;
                                stat.line = 0;
                            }
                            2 | 3 => {
                                //move on to setting comparator and immediate
                                // TODO: change to real task_primitive once we do stuff
                                stat.edit_leaf = match stat.line {
                                    2 => Filter::Leaf {
                                        field: "title".to_string(),
                                        comparator: Comparator::EQ,
                                        immediate: TaskPropVariant::String("".to_string()),
                                    },
                                    3 => Filter::Leaf {
                                        field: "title".to_string(),
                                        comparator: Comparator::EQ,
                                        immediate: TaskPropVariant::Boolean(true),
                                    },
                                    _ => unreachable!(),
                                };
                                stat.line = 0;
                                stat.state = CreateState::FilterConditional;
                            }
                            4..=7 => {
                                if !stat.is_editing {
                                    stat.is_editing = true;
                                } else {
                                    //disallow empty names
                                    if edit.is_empty() {
                                        err.replace_range(.., "field cannot be empty :)");
                                        return Ok(true);
                                    }

                                    //move on to next phase
                                    let imm = match stat.line {
                                        4 => TaskPropVariant::Number(0.0),
                                        5 => TaskPropVariant::String("".to_string()),
                                        7 => TaskPropVariant::Boolean(true),
                                        6 => TaskPropVariant::Date(NaiveDateTime::default()),
                                        _ => unreachable!(),
                                    };

                                    stat.edit_leaf = Filter::Leaf {
                                        field: edit.clone(),
                                        comparator: Comparator::EQ,
                                        immediate: imm,
                                    };
                                    stat.line = 0;
                                    stat.state = CreateState::FilterConditional;
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    (CreateState::FilterNameType, KeyCode::Esc) => {
                        if !stat.is_editing {
                            stat.state = CreateState::Filter;
                            stat.line = 0;
                        }
                        stat.is_editing = false;
                    }
                    (CreateState::FilterConditional, KeyCode::Enter) => {
                        let allowed = map_variant_to_allowed(stat.edit_leaf.clone());
                        if let Filter::Leaf {
                            field,
                            comparator: _,
                            immediate,
                        } = stat.edit_leaf.clone()
                        {
                            stat.edit_leaf = Filter::Leaf {
                                field,
                                comparator: allowed[stat.line].clone(),
                                immediate,
                            };

                            stat.line = 0;
                            edit.clear();
                            err.clear();
                            stat.state = CreateState::FilterPropImm;
                        }
                    }
                    (CreateState::FilterPropImm, KeyCode::Enter) => {
                        match stat.edit_leaf.clone() {
                            Filter::Leaf {
                                field: _,
                                comparator: _,
                                immediate,
                            } => {
                                let tpv: Option<TaskPropVariant> = match immediate {
                                    TaskPropVariant::Date(_) => {
                                        NaiveDateTime::parse_from_str(edit, "%Y-%m-%d %H:%M:%S")
                                            .map(Some)
                                            .unwrap_or(
                                                NaiveDate::parse_from_str(edit, "%Y-%m-%d")
                                                    .map(|a| a.and_hms_opt(0, 0, 0).unwrap())
                                                    .map(Some)
                                                    .unwrap_or(None),
                                            )
                                            .map(TaskPropVariant::Date)
                                    }
                                    TaskPropVariant::String(_) => {
                                        Some(TaskPropVariant::String(edit.clone()))
                                    }
                                    TaskPropVariant::Number(_) => edit
                                        .parse::<f64>()
                                        .map(Some)
                                        .unwrap_or(None)
                                        .map(TaskPropVariant::Number),
                                    TaskPropVariant::Boolean(_) => match edit.as_str() {
                                        "true" => Some(TaskPropVariant::Boolean(true)),
                                        "false" => Some(TaskPropVariant::Boolean(false)),
                                        _ => None,
                                    },
                                };

                                if let Some(x) = tpv {
                                    if let Filter::Leaf {
                                        field,
                                        comparator,
                                        immediate: _,
                                    } = stat.edit_leaf.clone()
                                    {
                                        //do stuff and leave :)
                                        let mut res: Vec<(String, &Filter, &Filter)> = vec![];
                                        traverse_filter(&mut res, &stat.filter, &Filter::None, 0);
                                        let bigroot = res.remove(0);

                                        let parent = if stat.line_to_add_to == res.len() {
                                            bigroot.1 // add to BIGROOT FRICK YEAH
                                        } else if let Filter::Leaf {
                                            field: _,
                                            comparator: _,
                                            immediate: _,
                                        } = res[stat.line_to_add_to].1
                                        {
                                            res[stat.line_to_add_to].2
                                        } else {
                                            res[stat.line_to_add_to].1
                                        };
                                        stat.filter = add_node_copy(
                                            &stat.filter,
                                            parent,
                                            &Filter::Leaf {
                                                field,
                                                comparator,
                                                immediate: x,
                                            },
                                        )
                                        .unwrap();
                                        stat.state = CreateState::Filter;
                                        stat.line = 0;
                                    }
                                } else {
                                    err.replace_range(
                                        ..,
                                        format!("{} could not be parsed", edit).as_str(),
                                    );
                                }
                            }
                            _ => unreachable!(),
                        };
                    }
                    (CreateState::FilterConditional | CreateState::FilterPropImm, KeyCode::Esc) => {
                        stat.state = CreateState::Filter;
                        stat.line = 0;
                    }
                    (_, KeyCode::Esc) => {
                        return Err(None);
                    }
                    (CreateState::Filter, KeyCode::Char(x)) => {
                        if *x == 'A' {
                            todo!("SEND IT")
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
                stat,
                err,
                name: _,
                props,
            } => match stat.state.clone() {
                CreateState::Name => {
                    let block = Block::default()
                        .title("Create View")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    Paragraph::new(format!("name: {}█\n{}", edit, err))
                        .block(block)
                        .render(area, buf);
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
                    text.push(Line::from(Span::raw(format!("{}█", edit))));
                    Paragraph::new(text).block(block).render(area, buf)
                }
                CreateState::Filter => {
                    let block = Block::default()
                        .title("Create Filter (hit `A` when you're happy)")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    let mut res: Vec<(String, &Filter, &Filter)> = vec![];
                    traverse_filter(&mut res, &stat.filter, &Filter::None, 0);
                    res.remove(0);

                    // clamp the index please
                    stat.line = min(max(0, stat.line), res.len() + 1);
                    let plus = "+".to_string();

                    let text: Vec<Line> = res
                        .iter()
                        .map(|(s, _, _)| s)
                        .chain([&plus])
                        .enumerate()
                        .map(|(i, s)| {
                            if i == stat.line {
                                Line::from(Span::styled(s, Style::new().red().bold()))
                            } else {
                                Line::from(Span::raw(s))
                            }
                        })
                        .chain([])
                        .collect();

                    Paragraph::new(text).block(block).render(area, buf);
                }
                CreateState::FilterNameType => {
                    let items = [
                        "AND".to_string(),
                        "OR".to_string(),
                        "title".to_string(),
                        "completed".to_string(),
                        format!("other: {} number", edit),
                        format!("       {} string", edit),
                        format!("       {} bool", edit),
                        format!("       {} date", edit),
                        err.to_string(),
                    ];
                    let block = Block::default()
                        .title("Field Name")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    let text: Vec<Line> = items
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            if i == stat.line && stat.is_editing {
                                Line::from(Span::styled(s, Style::new().green().bold()))
                            } else if i == stat.line {
                                Line::from(Span::styled(s, Style::new().red().bold()))
                            } else {
                                Line::from(Span::raw(s))
                            }
                        })
                        .collect();
                    Paragraph::new(text).block(block).render(area, buf);
                }
                CreateState::FilterConditional => {
                    let allowed = map_variant_to_allowed(stat.edit_leaf.clone());
                    let block = Block::default()
                        .title("Field Name")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);

                    let text: Vec<Line> = allowed
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            if i == stat.line {
                                Line::from(Span::styled(
                                    format!("{:?}", s),
                                    Style::new().red().bold(),
                                ))
                            } else {
                                Line::from(Span::raw(format!("{:?}", s)))
                            }
                        })
                        .collect();

                    Paragraph::new(text).block(block).render(area, buf);
                }
                CreateState::FilterPropImm => {
                    let text = vec![
                        Line::from(Span::raw(format!("imm: {}█", edit))),
                        Line::from(Span::raw(err.to_string())),
                    ];
                    let block = Block::default()
                        .title("Immediate")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED);
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
