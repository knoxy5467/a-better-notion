use num_modular::ModularCoreOps;
use ratatui::{buffer::Buffer, layout::Rect, style::{Modifier, Style}, text::{Line, Text}, widgets::{Block, HighlightSpacing, List, ListState, Paragraph, StatefulWidget, Widget}};

use crate::mid::{State, ViewKey};

use super::{COMPLETED_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};



#[derive(Default)]
/// Task list widget
pub struct TaskList {
    pub current_view: Option<ViewKey>,
    pub list_state: ListState,
}
impl TaskList {
    // move current selection of task up 1 item.
    pub fn up(&mut self, state: &State) {
        let Some(tasks) = self.current_view.and_then(|vk| state.view_tasks(vk)) else {
            self.list_state.select(None);
            return;
        };

        self.list_state.select(Some(
            self.list_state
                .selected()
                .as_mut()
                .map_or(0, |v| v.subm(1, &tasks.len())),
        ));
    }
    // move current selection of task down 1 item
    pub fn down(&mut self, state: &State) {
        let Some(tasks) = self.current_view.and_then(|vk| state.view_tasks(vk)) else {
            self.list_state.select(None);
            return;
        };
        self.list_state.select(Some(
            self.list_state
                .selected()
                .map_or(1, |v| v.addm(1, &tasks.len())),
        ));
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