use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, HighlightSpacing, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::mid::{State, Task, TaskKey, ViewKey};

use super::{COMPLETED_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};

#[derive(Debug)]
struct RenderedTask {
    key: TaskKey,
}

#[derive(Default, Debug)]
/// Task list widget
pub struct TaskList {
    pub list_state: ListState,
    /// views that we source the task list from
    source_views: Vec<ViewKey>,
    shown_tasks: Vec<TaskKey>,
}
impl TaskList {
    /// remove unused items
    pub fn prune_list(&mut self, state: &State) {
        let len = self.shown_tasks.len();
        // keep track of number of items removed so we can adjust selected item (if something is currently selected)
        let mut removed_count = 0;
        let mut did_switch = false;
        let current_task = self.list_state.selected().map(|s|self.shown_tasks[s]);
        self.shown_tasks.extract_if(|k|state.task_get(*k).is_ok()).for_each(|key| {
            if Some(key) == current_task { did_switch = true; };
            if !did_switch {
                removed_count += 1;
            }
        });
        // decrement current selected by amt_removed
        self.list_state.selected_mut().as_mut().map(|i| *i = (*i - removed_count).clamp(0, len));
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
    }
}
