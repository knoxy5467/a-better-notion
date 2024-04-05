//! Client
#![feature(coverage_attribute)]
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::{
    io::{self, stdout},
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

#[coverage(off)]
fn main() -> color_eyre::Result<()> {
    // manually create tokio runtime 
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(#[coverage(off)] async {
        initialize_logging()?;
        install_hooks()?;
        term::enable()?;
        let state = mid::init("http://localhost:8080").await?;
        let res = run(CrosstermBackend::new(stdout()), state, EventStream::new()).await;
        term::restore()?;
        res?;
        Ok(())
    })
}

#[coverage(off)]
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
#[coverage(off)]
pub fn install_hooks() -> color_eyre::Result<()> {
    // add any extra configuration you need to the hook builder
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    // used color_eyre's PanicHook as the standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(#[coverage(off)] move |panic_info| {
        term::restore().unwrap();
        panic_hook(panic_info);
    }));

    // use color_eyre's EyreHook as eyre's ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(#[coverage(off)] move |error| {
        term::restore().unwrap();
        eyre_hook(error)
    }))?;

    Ok(())
}

/// Run the program using writer, state, and event stream. abstracts between tests & main
async fn run<B: Backend>(backend: B, state: State, events: impl Stream<Item = io::Result<Event>> + Unpin) -> color_eyre::Result<App> {
    let mut term = Terminal::new(backend)?;
    let mut app = App::new(state);
    app.run(&mut term, events).await?;
    Ok(app)
}

/// UI App State
pub struct App {
    /// flag to be set to exit the event loop
    should_exit: bool,
    /// middleware state
    state: State,
    /// task list widget
    task_list: TaskList,
    /// number of frame updates (used for debug purposes)
    updates: usize,
    task_create_popup: Option<TaskCreatePopup>,
}

pub struct TaskCreatePopup {
    name: String,
    should_close: bool,
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
impl TaskCreatePopup {
    fn new() -> TaskCreatePopup {
        Self {
            name: Default::default(),
            should_close: false,
        }
    }
    fn render(&mut self, state: &State, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Create Task")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);
        let area = centered_rect(60, 20, area);
        let input = Paragraph::new(self.name.as_str()).block(block);
        Clear.render(area, buf);
        input.render(area, buf);
    }
    fn handle_key_event(&mut self, state: &mut State, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Esc => self.should_close = true,
            KeyCode::Char(c) => {
                self.name.push(c);
                
            }
            KeyCode::Backspace => {
                self.name.pop();
            }
            KeyCode::Enter => {
                let task_key = state.task_def(Task { name: self.name.clone(), ..Default::default() });
                state.view_mod(state.view_get_default().unwrap(), |v|v.tasks.as_mut().unwrap().push(task_key));
                self.should_close = true;
            }
            _ => return false,
        }
        true
    }
}

#[derive(Default)]
/// Task list widget
pub struct TaskList {
    current_view: Option<ViewKey>,
    list_state: ListState,
}
impl TaskList {
    // move current selection of task up 1 item.
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
    // move current selection of task down 1 item
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
    // render task list to buffer
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

impl App {
    /// create new app given middleware state
    pub fn new(state: State) -> Self {
        Self {
            should_exit: false,
            state,
            task_list: TaskList::default(),
            updates: 0,
            task_create_popup: None,
        }
    }
    /// run app with some terminal output and event stream input
    pub async fn run<B: Backend>(
        &mut self,
        term: &mut term::Tui<B>,
        mut events: impl Stream<Item = io::Result<Event>> + Unpin,
    ) -> color_eyre::Result<()> {
        self.task_list.current_view = self.state.view_get_default();
        // render initial frame
        term.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
        // wait for events
        while let Some(event) = events.next().await {
            // if we determined that event should trigger redraw:
            if self.handle_event(event?) {
                // draw frame
                term.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
            }
            // if we should exit, break loop
            if self.should_exit { break }
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            Event::Resize(_, _) => true,
            _ => false,
        }
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        use KeyCode::*;
        // handle if in popup state
        if let Some(task_create_popup) = &mut self.task_create_popup {
            return task_create_popup.handle_key_event(&mut self.state, key_event.code);
        }

        match key_event.code {
            Char('q') => self.should_exit = true,
            Char('e') => self.task_create_popup = Some(TaskCreatePopup::new()),
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
        self.updates += 1; // record render count
        
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
        // bottom right render update count
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

        if let Some(popup) = &mut self.task_create_popup {
            if popup.should_close { self.task_create_popup = None; }
            else {
                popup.render(&self.state, area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::SinkExt;
    use ratatui::backend::TestBackend;

    use super::*;

    #[tokio::test]
    async fn mock_app() {
        let backend = TestBackend::new(55, 5);
        let (mut sender, events) = futures::channel::mpsc::channel(10);

        let join = tokio::spawn(run(backend, init_test_state().0, events));

        // test regular event
        sender
            .send(Ok(Event::Key(KeyCode::Up.into())))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        // test non-rendering event
        sender
            .send(Ok(Event::Key(KeyCode::Char('1').into())))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        // test error event
        sender
            .send(Err(io::Error::other::<String>("error".into())))
            .await
            .unwrap();
        assert!(join.await.unwrap().is_err());

        let backend = TestBackend::new(55, 5);
        let (mut sender, events) = futures::channel::mpsc::channel(10);
        let join = tokio::spawn(run(backend, init_test_state().0, events));
        // test resize app
        sender
            .send(Ok(Event::Resize(0, 0)))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        // test quit app
        sender
            .send(Ok(Event::Key(KeyCode::Char('q').into())))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(join.await.unwrap().is_ok());

    }

    #[test]
    fn render_test() {
        // test default state
        let mut app = App::new(State::default());
        let mut buf = Buffer::empty(Rect::new(0, 0, 55, 5));

        app.render(buf.area, &mut buf);
        buf.set_style(Rect::new(0, 0, 55, 5), Style::reset());
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│              No Task Views to Display               │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 1╯",
        ]);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);

        // test task state
        let (state, view_key) = init_test_state();
        let mut app = App::new(state);
        app.task_list.current_view = Some(view_key);
        let mut buf = Buffer::empty(Rect::new(0, 0, 55, 5));

        app.render(buf.area, &mut buf);
        buf.set_style(Rect::new(0, 0, 55, 5), Style::reset());
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│  ☐ Finish ABN                                       │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 1╯",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> color_eyre::Result<()> {
        let mut app = App::new(State::default());
        // test up and down in example mid state
        let state = init_test_state();
        app.state = state.0;
        app.task_list.current_view = Some(state.1);
        app.handle_event(Event::Key(KeyCode::Up.into()));

        assert_eq!(app.task_list.list_state.selected(), Some(0));

        app.handle_event(Event::Key(KeyCode::Down.into()));
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
        app.handle_event(Event::Key(KeyCode::Up.into()));
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_event(Event::Key(KeyCode::Down.into()));
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_key_event(KeyCode::Enter.into());

        let mut app = App::new(State::default());
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.should_exit, true);

        let mut app = App::new(State::default());
        app.handle_key_event(KeyCode::Char('.').into());
        assert_eq!(app.should_exit, false);

        let mut app = App::new(State::default());
        app.handle_event(Event::FocusLost.into());
        assert_eq!(app.should_exit, false);

        Ok(())
    }
}
