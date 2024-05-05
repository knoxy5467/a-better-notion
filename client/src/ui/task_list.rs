mod task_popup;

use std::{collections::BTreeSet, iter::once};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, HighlightSpacing, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};

use crate::{mid::{PropNameKey, State, Task, TaskKey, ViewKey}, ui::{report_error, task_list::task_popup::CloseError}};

use task_popup::TaskPopup;

use super::{COMPLETED_TEXT_COLOR, GREYED_OUT_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};

#[derive(Default, Debug)]
/// Task list widget
pub struct TaskList {
    pub list_state: TableState,
    /// views that we source the task list from
    source_views: Vec<ViewKey>,
    /// tasks to display
    shown_tasks: Vec<TaskKey>,
    /// property types
    prop_cols: Vec<PropNameKey>,
    col_idx: usize,
    
    /// whether we are currently interacting with a task.
    task_popup: Option<TaskPopup>,
}
impl TaskList {
    /// remove unused items
    pub fn prune_list(&mut self, state: &State) {
        // keep track of number of items removed so we can adjust selected item (if something is currently selected)
        let mut removed_count = 0;
        let mut did_switch = false;
        let current_task = self.get_task_idx().and_then(|s|self.shown_tasks.get(s).cloned());
        self.shown_tasks.extract_if(|k|state.task_get(*k).is_err()).for_each(|key| {
            if Some(key) == current_task { did_switch = true; };
            if !did_switch {
                removed_count += 1;
            }
        });
        
        let len = self.shown_tasks.len();
        if len == 0 { // reset selection if neeeded
            self.set_task_idx(None);
        } else { // if not empty list
            // decrement current selected by amt_removed, ensuring selection is within span of list
            let new_idx = self.get_task_idx().map(|idx| {
                idx.saturating_sub(removed_count).clamp(0, len.saturating_sub(1))
            });
            self.set_task_idx(new_idx);
        }
        
    }
    /// update views that tasks to be shown are sourced from
    pub fn source_views_mod(&mut self, state: &State, func: impl FnOnce(&mut Vec<ViewKey>)) {
        func(&mut self.source_views);
        self.rebuild_list(state);
    } 
    /// recreate the shown list 
    pub fn rebuild_list(&mut self, state: &State) {
        let mut set = BTreeSet::new();
        // collect all items from source views into set
        self.source_views.iter().flat_map(|key|state.view_get(*key).ok())
            .map(|view|set.extend(view.tasks.iter().flatten())).last();
        // clear tasks and extend it with the generated set
        self.shown_tasks.clear();
        self.shown_tasks.extend(set.iter());
        self.set_task_idx(None); // clear selection
    }
    fn set_task_idx(&mut self, idx: Option<usize>) {
        self.list_state.select(idx.map(|i|i+1)); // add 1 to ignore table top bar
    }
    fn get_task_idx(&mut self) -> Option<usize> {
        self.list_state.selected().map(|i|i-1) // minus 1 so that calculations don't factor in table top bar
    }
    /// get currently selected task
    pub fn selected_task<'a>(&mut self, state: &'a State) -> Option<(TaskKey, &'a Task)> {
        self.prune_list(state);
        if self.shown_tasks.is_empty() { return None; } // error if no tasks
        self.get_task_idx().and_then(|r|{
            let key = self.shown_tasks[r];
            state.task_get(key).ok().map(|t|(key, t))
        })
    }
    // move current selection by amt in either direction, wrapping optionally
    pub fn shift(&mut self, amt: isize, wrap: bool) {
        let len = self.shown_tasks.len();
        // ensure we have at least 1 item
        if len == 0 { return; }

        // get current selected task, or if none currently selected, get last or first task depending on amt
        let cur_index = self.get_task_idx().unwrap_or(match amt.cmp(&0) {
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
        self.set_task_idx(Some(bounded_index)); // +1 to ignore top row (i.e. column name)
    }
    pub fn handle_term_event(&mut self, state: &mut State, event: &Event) -> bool {
        use KeyCode::*;
        // handle if in popup state
        if let Some(task_popup) = &mut self.task_popup {
            // early return if popup exists
            return match task_popup.handle_term_event(state, event) {
                Ok(do_render) => do_render,
                Err(err) => {
                    self.task_popup = None;
                    if let Some(err) = err {
                        match err {
                            CloseError::NoTaskError(err) => log::error!("attempted to delete a task: {err:?}"),
                            CloseError::ModifyTaskError(err) => log::error!("attempted to modify a task but got error: {err:?}"),
                            CloseError::AddTask(t) => self.shown_tasks.push(t),
                        }
                    }
                    true
                }
            }
        }
        let Event::Key(key_event) = event else {return false};
        match key_event.code {
            Char('c') => self.task_popup = Some(TaskPopup::Create(Default::default())), // create task
            Char('d') => { // delete task
                if let Some((key, task)) = self.selected_task(state) {
                    self.task_popup = Some(TaskPopup::Delete(key, task.name.clone()));
                }
            },
            Char('e') => {
                if let Some((selection, _)) = self.selected_task(state) {
                    self.task_popup = TaskPopup::edit(selection, state);
                }
            }
            Up => self.shift(-1, false),
            Down => self.shift(1, false),
            Enter => {
                if let Some((selected_key, _)) = self.selected_task(state) {
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
        self.prune_list(state);
        let valid_tasks = self.shown_tasks.iter().flat_map(|key|
            state.task_get(*key).ok().map(|t|(key, t))
        );

        // get selected column property name
        let prop_col_key = self.prop_cols.get(self.col_idx);
        // string form of key
        let prop_col_name = prop_col_key.map(|&key|state.prop_get_name(key).expect("invalid prop name key"));
        
        // table top column names
        // abusing iterator combinators be like: (as opposed to doing the sane thing and matching on prop_col_name)
        let top = Row::new(once("Tasks").chain(prop_col_name).collect::<Vec<&str>>());

        // Columns widths are constrained in the same way as Layout...
        let widths = [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ];

        // take items from the current view and render them into a list
        let mut data_rows = valid_tasks.map(|(task_key, task)| {
            let mut text_style: Style = if task.completed { COMPLETED_TEXT_COLOR.into() } else { TEXT_COLOR.into() };
            if !task.is_syncronized { text_style = GREYED_OUT_TEXT_COLOR.into(); }
            if task.pending_deletion { text_style = text_style.add_modifier(Modifier::CROSSED_OUT) }
            // completed mark
            let mut mark : &'static str = "☐";
            if task.completed { mark = "✓"; }
            // format prop
            let prop_data = prop_col_key.map(|name_key|{
                state.prop_get(*task_key, *name_key).ok().map(|prop|{
                    Span::raw(format!("{prop}"))
                }).unwrap_or(Span::default())
            });
            // create row, optionally with a prop of prop_key is not None
            Row::new(
                once(Span::styled(format!(" {mark} {}", task.name), text_style))
                    .chain(prop_data))
        }).peekable();
        if data_rows.peek().is_some() { // if there are tasks to render
            // create the list from the list items and customize it
            let table = Table::new(once(top).chain(data_rows), widths)
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
            StatefulWidget::render(table, area, buf, &mut self.list_state);
        } else { // otherwise render "no tasks shown" text
            let no_view_text = Text::from(vec![Line::from(vec!["No Tasks, Have you Selected a View?".into()])]);
            Paragraph::new(no_view_text)
                .centered()
                .block(block)
                .render(area, buf);
        }   
        // popup rendering
        if let Some(popup) = self.task_popup.as_mut() {
            popup.render(area, buf)
        }
    }
}
