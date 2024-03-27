//! Client

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::io;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::StreamExt;
use mid::*;
use num_modular::ModularCoreOps;
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
mod mid;
mod term;

const BACKGROUND: Color = Color::Reset;
const TEXT_COLOR: Color = Color::White;
const SELECTED_STYLE_FG: Color = Color::LightYellow;
const COMPLETED_TEXT_COLOR: Color = Color::Green;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut term = term::init()?;
    let res = App::default().run(&mut term).await;
    term::restore()?;
    res
}

/// UI App State
#[derive(Default)]
pub struct App {
    /// should exit
    should_exit: bool,
    /// middleware state
    state: State,
    /// task list widget
    task_list: TaskList,
    updates: usize,
}

#[derive(Default)]
/// Task list widget
pub struct TaskList {
    current_view: ViewKey,
    list_state: ListState,
}
impl TaskList {
    fn up(&mut self, state: &State) {
        let Some(tasks) = state.view_tasks(self.current_view) else {
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
    fn down(&mut self, state: &State) {
        let Some(tasks) = state.view_tasks(self.current_view) else {
            self.list_state.select(None);
            return;
        };
        self.list_state.select(Some(
            self.list_state
                .selected()
                .map_or(1, |v| v.addm(1, &tasks.len())),
        ));
    }
    fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        // take items from the current view and render them into a list
        if let Some(items) = state.view_tasks(self.current_view).map(|tasks| {
            tasks
                .iter()
                .flat_map(|key| {
                    let task = state.task_get(*key)?;
                    Some(match task.completed {
                        false => Line::styled(format!(" ☐ {}", task.name), TEXT_COLOR),
                        true => Line::styled(format!(" ✓ {}", task.name), COMPLETED_TEXT_COLOR),
                    })
                })
                .collect::<Vec<Line>>()
        }) {
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

impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, term: &mut term::Tui) -> io::Result<()> {
        let mut events = EventStream::new();

        // initialize state for testing
        let state = init_example();
        self.state = state.0;
        self.task_list.current_view = state.1;

        // main loop
        while !self.should_exit {
            self.updates += 1;
            term.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;

            let mut do_render = false;
            while !do_render {
                let Some(event) = events.next().await else {
                    continue;
                };
                do_render = self.handle_event(event?)?
            }
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_event(&mut self, event: Event) -> io::Result<bool> {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                Ok(self.handle_key_event(key_event))
            }
            Event::Resize(_, _) => Ok(true),
            _ => Ok(false),
        }
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        use KeyCode::*;
        match key_event.code {
            Char('q') => self.should_exit = true,
            Up => self.task_list.up(&self.state),
            Down => self.task_list.down(&self.state),
            Enter => {
                if let Some(selection) = self.task_list.list_state.selected() {
                    if let Some(tasks) = self.state.view_tasks(self.task_list.current_view) {
                        self.state
                            .task_mod(tasks[selection], |t| t.completed = !t.completed);
                    }
                }
            }
            _ => return false,
        }
        true // assume if didn't explicitly return false, that we should re-render
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Task Management ".bold());
        // bottom bar instructions
        let instructions = Title::from(Line::from(vec![
            " Select: ".into(),
            "<Up>".blue().bold(),
            "/".into(),
            "<Down>".blue().bold(),
            ", Quit: ".into(),
            "<Q> ".blue().bold(),
        ]));
        let update_counter = Title::from(format!("Updates: {}", self.updates));
        let block = Block::default()
            .bg(BACKGROUND)
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .title(
                update_counter
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        self.task_list.render(&self.state, block, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn dummy_test_main() {
        std::thread::spawn(main);
        std::thread::sleep(Duration::from_millis(250));
        term::restore().unwrap();
    }

    #[test]
    fn render() {
        let mut app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 7));

        app.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "╭─────────────── Task Management ────────────────╮",
            "│            No Task Views to Display            │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "╰──────── Select: <Up>/<Down>, Quit: <Q> ────────╯",
        ]);
        buf.set_style(Rect::new(0, 0, 50, 7), Style::reset());

        // don't bother checking styles, they change too frequently
        /*
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(16, 0, 17, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style); */

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        // test up and down in example mid state
        let state = init_example();
        app.state = state.0;
        app.task_list.current_view = state.1;
        app.handle_event(Event::Key(KeyCode::Up.into()))?;

        assert_eq!(app.task_list.list_state.selected(), Some(0));

        app.handle_event(Event::Key(KeyCode::Down.into()))?;
        assert_eq!(app.task_list.list_state.selected(), Some(1));

        // test enter key
        app.handle_key_event(KeyCode::Enter.into());
        assert_eq!(
            app.state
                .task_get(app.state.view_tasks(state.1).unwrap()[1])
                .unwrap()
                .completed,
            true
        ); // second task in example view is marked as completed, so the Enter key should uncomplete it

        // test up and down in regular state
        let mut app = App::default();
        app.handle_event(Event::Key(KeyCode::Up.into()))?;
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_event(Event::Key(KeyCode::Down.into()))?;
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_key_event(KeyCode::Enter.into());

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.should_exit, true);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('.').into());
        assert_eq!(app.should_exit, false);

        let mut app = App::default();
        app.handle_event(Event::FocusLost.into())?;
        assert_eq!(app.should_exit, false);

        Ok(())
    }
}
