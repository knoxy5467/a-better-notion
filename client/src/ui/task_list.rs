use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, HighlightSpacing, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::mid::{State, TaskKey, ViewKey};

use super::{COMPLETED_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};

#[derive(Default, Debug)]
/// Task list widget
pub struct TaskList {
    pub current_view: Option<ViewKey>,
    pub selected_task: Option<TaskKey>,
    pub list_state: ListState,
}
impl TaskList {
    fn reset_selection(&mut self) {
        self.list_state.select(None);
        self.selected_task = None;
    }
    // move current selection of task up 1 item.
    pub fn shift(&mut self, state: &State, amt: isize, wrap: bool) {
        // get tasks list (if any)
        let Some(tasks) = self.current_view.and_then(|vk| state.view_task_keys(vk)) else {
            self.reset_selection();
            return;
        };
        let tasks_iter = tasks.iter().filter(|key|state.task_get(**key).is_some());
        let len = tasks_iter.clone().count();
        // if empty, reset selection
        if len == 0 { self.reset_selection(); return; }
        
        let selected_task = self.selected_task.or_else(|| {
            let mut loc_tasks_iter = tasks_iter.clone();
            match amt.cmp(&0) {
                std::cmp::Ordering::Less => loc_tasks_iter.next_back().cloned(),
                std::cmp::Ordering::Greater => loc_tasks_iter.next().cloned(),
                _ => None, // return if amt is zero
            }
        });
        let Some(selected_task) = selected_task else { self.reset_selection(); return };

        // find first task before selected_task
        let mut cur_true_index = None;
        let mut cur_index = 0;
        for key in tasks {
            if let Some(_) = state.task_get(*key) {
                cur_true_index = Some(cur_index);
                cur_index += 1;
            }
            if *key == selected_task { break; }
        }
        let Some(cur_index) = cur_true_index else {
            return;
        };

        // add the index
        let new_index = (cur_index as isize) + amt;
        let new_index = if wrap {
            (len.saturating_add_signed(new_index)) % len
        } else {
            new_index.clamp(0, len.saturating_sub(1) as isize) as usize
        };
        self.list_state.select(Some(new_index));
        self.selected_task = tasks_iter.clone().nth(new_index).cloned();
    }
    // render task list to buffer
    pub fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        self.shift(state, 0, false);
        // No view available
        let mut no_view_text = Text::from(vec![Line::from(vec!["No Task Views to Display".into()])]);

        // take items from the current view and render them into a list
        if let Some(items) = self
            .current_view
            .and_then(|vk| state.view_tasks(vk))
            .map(|tasks| {
                tasks
                    .map(|(_key, task)| match task.completed {
                        false => Line::styled(format!(" ☐ {}", task.name), TEXT_COLOR),
                        true => Line::styled(format!(" ✓ {}", task.name), COMPLETED_TEXT_COLOR),
                    })
                    .collect::<Vec<Line>>()
            })
        {
            if !items.is_empty() {
                // create the list from the list items and customize it
                let list = List::new(items)
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
                return; // early return to prevent rendering default stuff
            } else {
                no_view_text = Text::from(vec![Line::from(vec!["No Tasks in View".into()])]);
            }
        } 
        Paragraph::new(no_view_text)
                .centered()
                .block(block)
                .render(area, buf);
    }
}
