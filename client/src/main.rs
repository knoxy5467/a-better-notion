//! Client

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::{
    io::{self, Write},
    panic,
};

use color_eyre::eyre;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{Stream, StreamExt};
use mid::*;
use num_modular::ModularCoreOps;
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
mod mid;
mod term;

const BACKGROUND: Color = Color::Reset;
const TEXT_COLOR: Color = Color::White;
const SELECTED_STYLE_FG: Color = Color::LightYellow;
const COMPLETED_TEXT_COLOR: Color = Color::Green;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    initialize_logging()?;
    install_hooks()?;
    let term = term::init(std::io::stdout())?;
    let res = run(term).await;
    term::restore()?;
    res
}
async fn run<W: io::Write>(mut term: term::Tui<W>) -> color_eyre::Result<()> {
    let state = mid::init("http://localhost:8080").await?;
    let events = EventStream::new();
    App::new(state).run(&mut term, events).await
}

fn initialize_logging() -> color_eyre::Result<()> {
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(io::stdout)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> color_eyre::Result<()> {
    // add any extra configuration you need to the hook builder
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        term::restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        term::restore().unwrap();
        eyre_hook(error)
    }))?;

    Ok(())
}

/// UI App State
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
    current_view: Option<ViewKey>,
    list_state: ListState,
}
impl TaskList {
    fn up(&mut self, state: &State) {
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
    fn down(&mut self, state: &State) {
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
    fn render(&mut self, state: &State, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        // take items from the current view and render them into a list
        if let Some(items) = self
            .current_view
            .and_then(|vk| state.view_tasks(vk))
            .map(|tasks| {
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

impl App {
    /// runs the application's main loop until the user quits
    pub fn new(state: State) -> Self {
        Self {
            should_exit: false,
            state,
            task_list: TaskList::default(),
            updates: 0,
        }
    }
    /// run app with some terminal output and event stream input
    pub async fn run<W: Write>(
        &mut self,
        term: &mut term::Tui<W>,
        mut events: impl Stream<Item = io::Result<Event>> + Unpin,
    ) -> color_eyre::Result<()> {
        self.task_list.current_view = self.state.view_get_default();
        // while not exist
        while !self.should_exit {
            self.updates += 1; // keep track of update & render
            term.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;

            // listen for evens and only re-render if we receive one that would imply we need to re-render
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
    fn handle_event(&mut self, event: Event) -> color_eyre::Result<bool> {
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
                    if let Some(tasks) = self
                        .task_list
                        .current_view
                        .and_then(|vk| self.state.view_tasks(vk))
                    {
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

    use futures::SinkExt;

    use super::*;

    #[test]
    fn dummy_test_main() {
        std::thread::spawn(main);
        std::thread::sleep(Duration::from_millis(250));
        term::restore().unwrap();
    }

    #[tokio::test]
    async fn mock_app() {
        let out = Box::leak(Box::new(Vec::new()));
        let writer = io::BufWriter::new(out);
        let (mut sender, events) = futures::channel::mpsc::channel(10);
        let join = tokio::spawn(async move {
            let mut term = term::init(writer).unwrap();
            let mut app = App::new(init_test_state().0);
            let res = app.run(&mut term, events).await;
            term::restore().unwrap();
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        sender
            .send(Ok(Event::Key(KeyCode::Up.into())))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        sender
            .send(Err(io::Error::other::<String>("error".into())))
            .await
            .unwrap();
        assert!(join.await.is_ok());
    }

    #[test]
    fn render_test() {
        let mut app = App::new(State::default());
        let mut buf = Buffer::empty(Rect::new(0, 0, 55, 5));

        app.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│              No Task Views to Display               │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 0╯",
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
    fn handle_key_event() -> color_eyre::Result<()> {
        let mut app = App::new(State::default());
        // test up and down in example mid state
        let state = init_test_state();
        app.state = state.0;
        app.task_list.current_view = Some(state.1);
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
        let mut app = App::new(State::default());
        app.handle_event(Event::Key(KeyCode::Up.into()))?;
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_event(Event::Key(KeyCode::Down.into()))?;
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_key_event(KeyCode::Enter.into());

        let mut app = App::new(State::default());
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.should_exit, true);

        let mut app = App::new(State::default());
        app.handle_key_event(KeyCode::Char('.').into());
        assert_eq!(app.should_exit, false);

        let mut app = App::new(State::default());
        app.handle_event(Event::FocusLost.into())?;
        assert_eq!(app.should_exit, false);

        Ok(())
    }
}
