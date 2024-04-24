mod task_popup;

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, HighlightSpacing, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{mid::{State, Task, TaskKey, ViewKey}, ui::{report_error, task_list::task_popup::CloseError}};

use task_popup::TaskPopup;

use super::{COMPLETED_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};

#[derive(Default, Debug)]
/// Task list widget
pub struct TaskList {
    pub list_state: ListState,
    /// views that we source the task list from
    source_views: Vec<ViewKey>,
    shown_tasks: Vec<TaskKey>,
    task_popup: Option<TaskPopup>,
}
impl TaskList {
    /// remove unused items
    pub fn prune_list(&mut self, state: &State) {
        let len = self.shown_tasks.len();
        // keep track of number of items removed so we can adjust selected item (if something is currently selected)
        let mut removed_count = 0;
        let mut did_switch = false;
        let current_task = self.list_state.selected().map(|s|self.shown_tasks.get(s).cloned()).flatten();
        self.shown_tasks.extract_if(|k|state.task_get(*k).is_err()).for_each(|key| {
            if Some(key) == current_task { did_switch = true; };
            if !did_switch {
                removed_count += 1;
            }
        });
        // decrement current selected by amt_removed
        self.list_state.selected_mut().as_mut().map(|i| *i = (i.saturating_sub(removed_count)).clamp(0, len));
    }
    /// update views that tasks to be shown are sourced from
    pub fn source_views_mod(&mut self, state: &State, func: impl FnOnce(&mut Vec<ViewKey>)) {
        func(&mut self.source_views);
        self.rebuild_list(state);
    } 
    /// recreate the shown list 
    pub fn rebuild_list(&mut self, state: &State) {
        let mut set = HashSet::new();
        // collect all items from source views into set
        self.source_views.iter().flat_map(|key|state.view_get(*key).ok())
            .map(|view|set.extend(view.tasks.iter().flatten())).last();
        // clear tasks and extend it with the generated set
        self.shown_tasks.clear();
        self.shown_tasks.extend(set.iter());
        self.list_state.select(None); // clear selection
    }
    /// get currently selected task
    pub fn selected_task<'a>(&mut self, state: &'a State) -> Option<(TaskKey, &'a Task)> {
        self.prune_list(state);
        if self.shown_tasks.len() == 0 { return None; } // error if no tasks
        self.list_state.selected().map(|r|{
            let key = self.shown_tasks[r];
            state.task_get(key).ok().map(|t|(key, t))
        }).flatten()
    }
    // move current selection by amt in either direction, wrapping optionally
    pub fn shift(&mut self, amt: isize, wrap: bool) {
        let len = self.shown_tasks.len();
        // ensure we have at least 1 item
        if len == 0 { return; }

        // get current selected task, or if none currently selected, get last or first task depending on amt
        let cur_index = self.list_state.selected().unwrap_or(match amt.cmp(&0) {
            std::cmp::Ordering::Less => len.saturating_sub(1),
            std::cmp::Ordering::Greater => 0,
            _ => return, // early return if amt=0
        });

        // add amt to current index
        let new_index = (cur_index as isize) + amt;
        // ensure new_index is in bounds
        let bounded_index = if wrap { // if wrapping, add signed and modulo length
            (len.saturating_add_signed(new_index)) % len
        } else { // if not wrapping, clamp to bounds
            new_index.clamp(0, len.saturating_sub(1) as isize) as usize
        };
        // set selection
        self.list_state.select(Some(bounded_index));
    }
    pub fn handle_key_event(&mut self, state: &mut State, key_event: KeyEvent) -> bool {
        use KeyCode::*;
        // handle if in popup state
        if let Some(task_popup) = &mut self.task_popup {
            match task_popup.handle_key_event(state, key_event.code) {
                Ok(do_render) => return do_render,
                Err(err) => {
                    self.task_popup = None;
                    if let Some(err) = err {
                        match err {
                            CloseError::NoTask(err) => log::error!("attempted to delete a task that does not exist: {err:?}"),
                            CloseError::AddTask(t) => self.shown_tasks.push(t),
                        }
                    }
                }
            }
        }
        match key_event.code {
            Char('c') => self.task_popup = Some(TaskPopup::Create(Default::default())), // create task
            Char('d') => { // delete task
                if let Some((key, task)) = self.selected_task(&state) {
                    self.task_popup = Some(TaskPopup::Delete(key, task.name.clone()));
                }
            },
            /* Char('e') => {
                if let Some(selection) = self.task_list.selected_task {
                    self.taks = Some(TaskEditPopup::new(Some(selection)));
                }
            } */
            Up => self.shift(-1, false),
            Down => self.shift(1, false),
            Enter => {
                if let Some((selected_key, _)) = self.selected_task(&state) {
                    let res = state.task_mod(selected_key, |t| t.completed = !t.completed);
                    if let Err(err) = res {
                        report_error(err);
                    }
                }
            }
            _ => return false,
        }
        true // assume if didn't explicitly return false, that we should re-render
    }
    // render task list to buffer
    pub fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        // flat_map current tasks to make sure they're valid
        let valid_tasks = self.shown_tasks.iter().flat_map(|key|
            state.task_get(*key).ok().map(|t|(key, t))
        );

        // take items from the current view and render them into a list
        let lines = valid_tasks.map(|(_key, task)| match task.completed {
            false => Line::styled(format!(" ☐ {}", task.name), TEXT_COLOR),
            true => Line::styled(format!(" ✓ {}", task.name), COMPLETED_TEXT_COLOR),
        })
        .collect::<Vec<Line>>();

        if !lines.is_empty() { // if there are tasks to render
            // create the list from the list items and customize it
            let list = List::new(lines)
                .block(block)
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                        .fg(SELECTED_STYLE_FG),
                )
                .highlight_symbol(">")
                .highlight_spacing(HighlightSpacing::Always);

            // render the list using the list state
            StatefulWidget::render(list, area, buf, &mut self.list_state);
        } else { // otherwise render "no tasks shown" text
            let no_view_text = Text::from(vec![Line::from(vec!["No Tasks, Have you Selected a View?".into()])]);
            Paragraph::new(no_view_text)
                .centered()
                .block(block)
                .render(area, buf);
        }   
        // popup rendering
        self.task_popup.as_ref().inspect(|t|t.render(area, buf));
    }
}
