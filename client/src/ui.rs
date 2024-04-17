use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use futures::{channel::mpsc::Receiver, Stream, StreamExt};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use crate::{mid::{MidEvent, State, StateEvent}, term};

use self::task_create_popup::TaskCreatePopup;
use self::task_delete_popup::TaskDeletePopup;

mod task_create_popup;
mod task_delete_popup;
mod task_list;

const BACKGROUND: Color = Color::Reset;
const TEXT_COLOR: Color = Color::White;
const SELECTED_STYLE_FG: Color = Color::LightYellow;
const COMPLETED_TEXT_COLOR: Color = Color::Green;

/// Run the program using writer, state, and event stream. abstracts between tests & main
pub async fn run<B: Backend>(
    backend: B,
    state: (State, Receiver<MidEvent>),
    events: impl Stream<Item = io::Result<Event>> + Unpin,
) -> color_eyre::Result<App> {
    let mut term = Terminal::new(backend)?;
    let mut app = App::new(state.0);
    app.run(&mut term, events, state.1).await?;
    Ok(app)
}

/// UI App State
#[derive(Debug)]
pub struct App {
    /// flag to be set to exit the event loop
    should_exit: bool,
    /// middleware state
    state: State,
    /// task list widget
    task_list: task_list::TaskList,
    /// number of frame updates (used for debug purposes)
    updates: usize,
    task_create_popup: Option<TaskCreatePopup>,
    task_delete_popup: Option<TaskDeletePopup>
}

pub enum UIEvent {
    UserEvent(Event),
    StateEvent(StateEvent),
}

impl App {
    /// create new app given middleware state
    pub fn new(state: State) -> Self {
        Self {
            should_exit: false,
            state,
            task_list: task_list::TaskList::default(),
            updates: 0,
            task_create_popup: None,
            task_delete_popup: None,
        }
    }
    /// run app with some terminal output and event stream input
    pub async fn run<B: Backend>(
        &mut self,
        term: &mut term::Tui<B>,
        mut events: impl Stream<Item = io::Result<Event>> + Unpin,
        mut state_events: impl Stream<Item = MidEvent> + Unpin,
    ) -> color_eyre::Result<()> {
        self.task_list.current_view = self.state.view_get_default();
        // render initial frame
        term.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
        // wait for events
        loop {
            tokio::select! {
                Some(event) = events.next() => self.step(term, UIEvent::UserEvent(event?))?,
                Some(mid_event) = state_events.next() => if let MidEvent::StateEvent(state_event) = mid_event {
                    self.step(term, UIEvent::StateEvent(state_event))?;
                } else { // else handle middleware event
                    self.state.handle_mid_event(mid_event)?;
                },
                else => break,
            }
            if self.should_exit {
                break;
            }
        }
        /* while let Some(event) = events.next().await {
            self.step(term, event?)?;
            // if we should exit, break loop
            
        } */
        Ok(())
    }
    pub fn step<B: Backend>(
        &mut self,
        term: &mut term::Tui<B>,
        event: UIEvent,
    ) -> color_eyre::Result<()> {
        // if we determined that event should trigger redraw:
        if self.handle_event(event) {
            // draw frame
            term.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_event(&mut self, event: UIEvent) -> bool {
        match event {
            UIEvent::UserEvent(event) => match event {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                Event::Resize(_, _) => true,
                _ => false,
            },
            UIEvent::StateEvent(state_event) => match state_event {
                StateEvent::TaskUpdate(_) => todo!(),
                StateEvent::PropUpdate(_) => todo!(),
                StateEvent::ViewUpdate(_) => todo!(),
                StateEvent::ScriptUpdate(_) => todo!(),
                StateEvent::MultiState => true,
                StateEvent::ServerStatus(_) => todo!(),
            },
        }
        
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        use KeyCode::*;
        // handle if in popup state
        if let Some(task_create_popup) = &mut self.task_create_popup {
            return task_create_popup.handle_key_event(&mut self.state, key_event.code);
        }
        if let Some(task_delete_popup) = &mut self.task_delete_popup {
            return task_delete_popup.handle_key_event(&mut self.state, key_event.code);
        } 

        match key_event.code {
            Char('q') => self.should_exit = true,
            Char('c') => self.task_create_popup = Some(TaskCreatePopup::new()),
            Up => self.task_list.shift(&self.state, -1, false),
            Down => self.task_list.shift(&self.state, 1, false),
            Char('d') => {
                if let Some(selection) = self.task_list.selected_task {
                    self.task_delete_popup = Some(TaskDeletePopup::new(selection));
                }
            }
            Enter => {
                if let Some(selection) = self.task_list.list_state.selected() {
                    if let Some(tasks) = self
                        .task_list
                        .current_view
                        .and_then(|vk| self.state.view_task_keys(vk))
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
            " Help: ".into(),
            "<H> ".blue().bold(),
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

        if let Some(task_create_popup) = &mut self.task_create_popup {
            if task_create_popup.should_close {
                self.task_create_popup = None;
            } else {
                task_create_popup.render(area, buf);
            }
        }

        if let Some(task_delete_popup) = &mut self.task_delete_popup {
            if task_delete_popup.should_close {
                self.task_delete_popup = None;
            }
            else {
                task_delete_popup.render(area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::SinkExt;
    use ratatui::backend::TestBackend;

    use crate::{mid::init_test, ui::UIEvent::UserEvent};

    use super::*;

    #[tokio::test]
    async fn mock_app() {
        let backend = TestBackend::new(55, 5);
        let (mut sender, events) = futures::channel::mpsc::channel(10);

        let join = tokio::spawn(run(backend, init_test(), events));

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
        let join = tokio::spawn(run(backend, init_test(), events));
        // test resize app
        sender.send(Ok(Event::Resize(0, 0))).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        // test quit app
        sender
            .send(Ok(Event::Key(KeyCode::Char('q').into())))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(join.await.unwrap().is_ok());
    }

    fn create_render_test(state: State, width: u16, height: u16) -> (App, term::Tui<TestBackend>) {
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut app = App::new(state);
        term.draw(|f| f.render_widget(&mut app, f.size())).unwrap();
        (app, term)
    }
    fn reset_buffer_style(term: &mut term::Tui<TestBackend>) {
        let mut buffer_copy = term.backend().buffer().clone();
        buffer_copy.set_style(buffer_copy.area().clone(), Style::reset());
        let iter = buffer_copy
            .content()
            .iter()
            .enumerate()
            .map(|(i, c)| (buffer_copy.pos_of(i), c))
            .map(|((x, y), c)| (x, y, c));
        term.backend_mut().draw(iter).unwrap();
    }

    #[test]
    fn render_test() -> color_eyre::Result<()> {
        // test default state
        let (_, mut term) = create_render_test(State::new().0, 55, 5);

        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│              No Task Views to Display               │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 1╯",
        ]);
        term.backend_mut().assert_buffer(&expected);

        // test task state
        let (state, _) = init_test();
        let (mut app, mut term) = create_render_test(state, 55, 5);
        app.task_list.current_view = app.state.view_get_default(); // set the view key as is currently done in run()
        println!("{:?}", app);

        app.step(&mut term, UserEvent(Event::Key(KeyCode::Down.into())))?;
        println!("{:#?}", app);
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ Finish ABN                                       │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 2╯",
        ]);
        term.backend().assert_buffer(&expected);

        // resize
        term.backend_mut().resize(55, 8);
        app.step(&mut term, UserEvent(Event::Resize(55, 88)))?;
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ Finish ABN                                       │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 3╯",
        ]);
        term.backend().assert_buffer(&expected);

        // test task creation
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Char('e').into())))?;
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Char('h').into())))?;
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Char('i').into())))?;
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Char('!').into())))?;
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Backspace.into())))?;
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ Finish ABN                                       │",
            "│             ╭Create Task──────────────╮             │",
            "│             │hi                       │             │",
            "│             ╰─────────────────────────╯             │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> ─Updates: 8╯",
        ]);
        term.backend().assert_buffer(&expected);

        app.step(&mut term, UserEvent(Event::Key(KeyCode::Enter.into())))?;
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Char('e').into())))?;
        app.step(&mut term, UserEvent(Event::Key(KeyCode::Esc.into())))?;
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ Finish ABN                                       │",
            "│  ☐ hi                                               │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> Updates: 11╯",
        ]);
        term.backend().assert_buffer(&expected);
        Ok(())
    }

    #[test]
    fn handle_key_event() -> color_eyre::Result<()> {
        let mut app = App::new(State::new().0);
        // test up and down in example mid state
        let (state, _) = init_test();
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());
        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));

        assert_eq!(app.task_list.list_state.selected(), Some(0));

        app.handle_event(UserEvent(Event::Key(KeyCode::Down.into())));
        assert_eq!(app.task_list.list_state.selected(), Some(1));

        // test enter key
        app.handle_key_event(KeyCode::Enter.into());
        assert_eq!(
            app.state
                .task_get(
                    app.state
                        .view_task_keys(app.state.view_get_default().unwrap())
                        .unwrap()[1]
                )
                .unwrap()
                .completed,
            true
        ); // second task in example view is marked as completed, so the Enter key should uncomplete it

        // test up and down in regular state
        let mut app = App::new(State::new().0);
        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_event(UserEvent(Event::Key(KeyCode::Down.into())));
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_key_event(KeyCode::Enter.into());

        let mut app = App::new(State::new().0);
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.should_exit, true);

        let mut app = App::new(State::new().0);
        app.handle_key_event(KeyCode::Char('.').into());
        assert_eq!(app.should_exit, false);

        let mut app = App::new(State::new().0);
        app.handle_event(UserEvent(Event::FocusLost.into()));
        assert_eq!(app.should_exit, false);

        Ok(())
    }
}
