use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    symbols::border,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, HighlightSpacing, List, ListState, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::mid::{ State, View, ViewKey };

use super::{COMPLETED_TEXT_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};

#[derive(Default, Debug)]
pub struct ViewList {
    pub selected_view: Option<ViewKey>,
    pub list_state: ListState,
}

impl ViewList {
    pub fn render(
        &mut self,
        state: &State,
        views: &Vec<ViewKey>,
        is_toggled: &HashMap<ViewKey, bool>,
        block: Block<'_>, area: Rect,
        buf: &mut Buffer
    ) {
        let items = views.iter().map(|view_key| {
            let view = state.view_get(*view_key).unwrap();
            match is_toggled[view_key] {
                true  => Line::styled(format!(" ☐ {}", view.name), TEXT_COLOR),
                false => Line::styled(format!(" ■ {}", view.name), COMPLETED_TEXT_COLOR),
            } 
        }).collect::<Vec<Line>>();

        // should always have at least the default view
        assert!(!items.is_empty());

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
        
    }
}

#[derive(Debug)]
pub struct ViewsPopup {
    view_list: ViewList,
    views: Vec<ViewKey>,
    is_toggled: HashMap<ViewKey, bool>,
    pub should_close: bool,
}
impl ViewsPopup {
    pub fn new(views: Vec<ViewKey>, is_toggled: HashMap<ViewKey, bool>) -> ViewsPopup {
        Self {
            view_list: ViewList::default(),
            views: views,
            is_toggled: is_toggled,
            should_close: false,
        }
    }
    
    pub fn render(&mut self, area: Rect, state: &State, buf: &mut Buffer) {
        let block = Block::default()
            .title("Views Manager")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        self.view_list.render(state, &self.views, &self.is_toggled, block, area, buf);
    }
    
    pub fn handle_key_event(&mut self, state: &mut State, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Esc       => self.should_close = true,
            KeyCode::Char('v') => self.should_close = true,
            KeyCode::Enter => {
                todo!();
            }
            _ => return false,
        }
        true
    }
}
