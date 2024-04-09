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
    // move current selection of task up 1 item.
    pub fn shift(&mut self, state: &State, amt: isize, wrap: bool) {
        // get tasks list (if any)
        let Some(tasks) = self.current_view.and_then(|vk| state.view_tasks(vk)) else {
            self.list_state.select(None);
            return;
        };
        // get current selected task (if any)
        let Some(selected_task) = self.selected_task.or(match amt.cmp(&0) {
            std::cmp::Ordering::Less => tasks.last().cloned(),
            std::cmp::Ordering::Greater => tasks.first().cloned(),
            _ => None,
        }) else { return; };
        // get index in tasks list of selected_task
        let Some(cur_index) = tasks.iter().position(|key|*key == selected_task) else {return;};
        // add the index
        let new_index = (cur_index as isize) + amt;
        let new_index = if wrap {
            (tasks.len().saturating_add_signed(new_index)) % tasks.len()
        } else {
            new_index.clamp(0, tasks.len().saturating_sub(1) as isize) as usize
        };
        self.list_state.select(Some(new_index));
        self.selected_task = Some(tasks[new_index]);
    }
    // render task list to buffer
    pub fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        // take items from the current view and render them into a list
        if let Some(items) = self
            .current_view
            .and_then(|vk| state.view_tasks(vk))
            .map(|tasks| {
                tasks
                    .iter()
                    .flat_map(|key| {
                        let task = state.task_get(*key)?;
                        // render task line
                        Some(match task.completed {
                            false => Line::styled(format!(" ☐ {}", task.name), TEXT_COLOR),
                            true => Line::styled(format!(" ✓ {}", task.name), COMPLETED_TEXT_COLOR),
                        })
                    })
                    .collect::<Vec<Line>>()
            })
        {
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
            StatefulWidget::render(list, area, buf, &mut self.list_state)
        } else {
            // No view available
            let no_view_text =
                Text::from(vec![Line::from(vec!["No Task Views to Display".into()])]);

            Paragraph::new(no_view_text)
                .centered()
                .block(block)
                .render(area, buf);
        }
    }
}
