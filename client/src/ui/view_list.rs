mod view_popup;

use std::collections::BTreeSet;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, HighlightSpacing, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{mid::{State, Task, TaskKey, ViewKey}, ui::{report_error}};

//use task_popup::TaskPopup;

use self::view_popup::ViewPopup;

use super::{COMPLETED_TEXT_COLOR, GREYED_OUT_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};

#[derive(Default, Debug)]
/// Task list widget
pub struct ViewList {
    pub list_state: ListState,
    all_views: Vec<ViewKey>,
    view_popup: Option<ViewPopup>,
}

impl ViewList {
    /// remove unused items
    // pub fn prune_list(&mut self, state: &State) {
    //     // keep track of number of items removed so we can adjust selected item (if something is currently selected)
    //     let mut removed_count = 0;
    //     let mut did_switch = false;
    //     let current_view = self.list_state.selected().and_then(|s|self.all_views.get(s).cloned());
    //     self.all_views.extract_if(|k|state.view_get(*k).is_err()).for_each(|key| {
    //         if Some(key) == current_task { did_switch = true; };
    //         if !did_switch {
    //             removed_count += 1;
    //         }
    //     });
        
    //     let len = self.all_views.len();
    //     if len == 0 { // reset selection if needed
    //         self.list_state.select(None);
    //     } else { // if not empty list
    //         // decrement current selected by amt_removed, ensuring selection is within span of list
    //         if let Some(i) = self.list_state.selected_mut().as_mut() {
    //             *i = (i.saturating_sub(removed_count)).clamp(0, len.saturating_sub(1))
    //         }
    //     }
        
    // }
    /// recreate view list
    pub fn rebuild_list(&mut self, state: &State) {
        self.all_views = state.view_get_keys().collect::<Vec<ViewKey>>();
    }
    /// get currently selected task
    // pub fn selected_view<'a>(&mut self, state: &'a State) -> Option<(ViewKey, &'a View)> {
    //     self.prune_list(state);
    //     self.list_state.selected().and_then(|i|{
    //         let key = self.all_views[i];
    //         state.view_get(key).ok().map(|v|(key, v))
    //     })
    // }
    // move current selection by amt in either direction, wrapping optionally
    // pub fn shift(&mut self, amt: isize, wrap: bool) {
    //     let len = self.all_views.len();
    //     // ensure we have at least 1 item
    //     if len == 0 { return; }

    //     // get current selected task, or if none currently selected, get last or first task depending on amt
    //     let cur_index = self.list_state.selected().unwrap_or(match amt.cmp(&0) {
    //         std::cmp::Ordering::Less => len.saturating_sub(1),
    //         std::cmp::Ordering::Greater => 0,
    //         _ => return, // early return if amt=0
    //     });

    //     // add amt to current index
    //     let new_index = (cur_index as isize) + amt;
    //     // ensure new_index is in bounds
    //     let bounded_index = if wrap { // if wrapping, add signed and modulo length
    //         (len.saturating_add_signed(new_index)) % len
    //     } else { // if not wrapping, clamp to bounds
    //         new_index.clamp(0, len.saturating_sub(1) as isize) as usize
    //     };
    //     // set selection
    //     self.list_state.select(Some(bounded_index));
    // }
    pub fn handle_term_event(&mut self, state: &mut State, event: &Event) -> bool {
        use KeyCode::*;
        // TODO: popups
        if let Some(view_popup) = &mut self.view_popup {
            // early return if popup exists
            return match view_popup.handle_term_event(state, event) {
                Ok(do_render) => do_render,
                Err(err) => {
                    self.view_popup = None;
                    if let Some(err) = err {
                        match err {
                            // CloseError::NoTaskError(err) => log::error!("attempted to delete a task: {err:?}"),
                            // CloseError::ModifyTaskError(err) => log::error!("attempted to modify a task but got error: {err:?}"),
                            view_popup::ViewPopupCloseError::AddView(v) => self.all_views.push(v),
                        }
                    }
                    true
                }
            }
        }
        let Event::Key(key_event) = event else {return false};
        // TODO: handle popup creates
        match key_event.code {
            Char('c') => self.view_popup = Some(ViewPopup::Create(Default::default())), // create task
            // Char('d') => { // delete task
            //     if let Some((key, task)) = self.selected_task(state) {
            //         self.task_popup = Some(TaskPopup::Delete(key, task.name.clone()));
            //     }
            // },
            // Char('e') => {
            //     if let Some((selection, _)) = self.selected_task(state) {
            //         self.task_popup = TaskPopup::edit(selection, state);
            //     }
            // }
            // Char('v') => {
            //     todo!();
            // }
            // Up => self.shift(-1, false),
            // Down => self.shift(1, false),
            // Enter => {
            //     if let Some((selected_key, _)) = self.selected_task(state) {
            //         let res = state.task_mod(selected_key, |t| t.completed = !t.completed);
            //         if let Err(err) = res {
            //             report_error(err);
            //         }
            //     }
            // }
            _ => return false,
        }
        true // assume if didn't explicitly return false, that we should re-render
    }
    // render task list to buffer
    pub fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        // flat_map current views to make sure they're valid
        //self.prune_list(state); // TODO
        let views = self.all_views.iter().flat_map(|key|
            state.view_get(*key).ok().map(|t|(key, t))
        );

        // take items from the current view and render them into a list
        let lines = views.map(|(_key, view)| {
            let mut text_style: Style = TEXT_COLOR.into();
            // if !task.is_syncronized { text_style = GREYED_OUT_TEXT_COLOR.into(); }
            // if task.pending_deletion { text_style = text_style.add_modifier(Modifier::CROSSED_OUT) }

            let mut mark : &'static str = "☐";
            // if task.completed { mark = "✓"; }

            return Line::styled(format!(" {mark} {}", view.name), text_style);
        })
        .collect::<Vec<Line>>();

        if !lines.is_empty() { // if there are views to render
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
            let no_view_text = Text::from(vec![Line::from(vec!["No Views. Create one, if you wish...".into()])]);
            Paragraph::new(no_view_text)
                .centered()
                .block(block)
                .render(area, buf);
        }   
        // popup rendering
        if let Some(popup) = self.view_popup.as_mut() {
            popup.render(area, buf)
        }
    }
}
