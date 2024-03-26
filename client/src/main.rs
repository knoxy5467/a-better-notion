//! Client

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use mid::*;
use num_modular::ModularCoreOps;
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
mod mid;
mod term;

const BACKGROUND: Color = Color::DarkGray;
const TEXT_COLOR: Color = Color::White;
const SELECTED_STYLE_FG: Color = Color::LightYellow;
const COMPLETED_TEXT_COLOR: Color = Color::Green;

fn main() -> io::Result<()> {
    term::wrap_terminal(|term| App::default().run(term))
}

/// UI App State
#[derive(Default)]
pub struct App {
    exit: bool, // should exit
    state: State, // middleware state
    task_list: TaskList,
}

#[derive(Default)]
/// Task list widget
pub struct TaskList {
    current_view: ViewKey,
    list_state: ListState,
}
impl TaskList {
    fn up(&mut self, state: &State) {
        let Some(tasks) = state.view_tasks(self.current_view)
        else { self.list_state.select(None); return; };

        self.list_state.select(Some(self.list_state.selected().as_mut().map_or(0, |v|v.subm(1, &tasks.len()))));
    }
    fn down(&mut self, state: &State) {
        let Some(tasks) = state.view_tasks(self.current_view)
        else { self.list_state.select(None); return; };
        self.list_state.select(Some(self.list_state.selected().map_or(1, |v|v.addm(1, &tasks.len()))));
    }
    fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        let items = state.view_tasks(self.current_view).and_then(|tasks| Some(tasks.iter().flat_map(|key| {
            let Some(task) = state.task_get(*key)
            else { return None };

            Some(match task.completed {
                false => Line::styled(format!(" ☐ {}", task.name), TEXT_COLOR),
                true => Line::styled(format!(" ✓ {}", task.name), COMPLETED_TEXT_COLOR),
            })
        }).collect::<Vec<Line>>()));

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

        StatefulWidget::render(list, area, buf, &mut self.list_state)
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut term::Tui) -> io::Result<()> {
        // initialize state for testing
        let state = init_example();
        self.state = state.0;
        self.task_list.current_view = state.1;

        // main loop
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
            self.handle_event(event::read()?)?;
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_event(&mut self, event: Event) -> io::Result<()> {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        use KeyCode::*;
        match key_event.code {
            Char('q') => self.exit = true,
            Up => self.task_list.up(&self.state),
            Down => self.task_list.down(&self.state),
            Enter => {
                if let Some(selection) = self.task_list.list_state.selected() {
                    if let Some(tasks) = self.state.view_tasks(self.task_list.current_view) {
                        self.state.task_mod(tasks[selection], |t|t.completed = !t.completed);
                    }
                }
            }
            _ => {}
        }
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
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        self.task_list.render(&self.state, block, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn dummy_test_main() {
        std::thread::spawn(|| main());
        std::thread::sleep(Duration::from_millis(250));
    }

    #[test]
    fn render() {
        let mut app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 7));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━ Task Management ━━━━━━━━━━━━━━━━┓",
            "┃                     Usage:                     ┃",
            "┃            Press <Button> To Do <X>            ┃",
            "┃         Press <Other Button> To Do <Y>         ┃",
            "┃               ↑ ↑ ↓ ↓ ← → ← → B A              ┃",
            "┃                   Level: 9001                  ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(16, 0, 17, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.exit, true);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('.').into());
        assert_eq!(app.exit, false);

        let mut app = App::default();
        app.handle_event(Event::FocusLost.into())?;
        assert_eq!(app.exit, false);

        Ok(())
    }
}
