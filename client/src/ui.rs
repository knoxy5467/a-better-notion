use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use futures::{channel::mpsc::Receiver, Stream, StreamExt};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
// use tokio::runtime::Handle;

use crate::{mid::{MidEvent, State, StateEvent}, term};

mod task_popup;
mod task_list;

use task_popup::TaskPopup;

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
    task_popup: Option<TaskPopup>,
    help_box_shown: bool,
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
            task_popup: None,
            help_box_shown: false,
        }
    }
    /// run app with some terminal output and event stream input
    pub async fn run<B: Backend>(
        &mut self,
        term: &mut term::Tui<B>,
        mut events: impl Stream<Item = io::Result<Event>> + Unpin,
        mut state_events: impl Stream<Item = MidEvent> + Unpin,
    ) -> color_eyre::Result<()> {
        self.task_list.source_views_mod(&self.state, |s|s.extend(self.state.view_get_default()));
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
                StateEvent::TasksUpdate => todo!(),
                StateEvent::PropsUpdate => todo!(),
                StateEvent::ViewsUpdate => {
                    self.task_list.rebuild_list(&self.state); // rebuild list state when views update
                    true
                },
                StateEvent::ScriptUpdate(_) => todo!(),
                StateEvent::ServerStatus(_) => todo!(),
            },
        }
        
    }
    fn report_error(&mut self, error: impl std::error::Error + std::fmt::Debug) {
        tracing::error!("{error}");
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        use KeyCode::*;
        // handle if in popup state
        if let Some(task_popup) = &mut self.task_popup {
            match task_popup.handle_key_event(&mut self.state, key_event.code) {
                Ok(do_render) => return do_render,
                Err(err) => {
                    self.task_popup = None;
                    err.map(|err|self.report_error(err));
                }
            }
        }
        if let Char('h') = key_event.code {} else { self.help_box_shown = false; }
        match key_event.code {
            Esc => if self.help_box_shown { self.help_box_shown = false; }
            Char('q') => self.should_exit = true,
            Char('h') => self.help_box_shown = !self.help_box_shown,
            Char('c') => self.task_popup = Some(TaskPopup::Create(Default::default())), // create task
            Char('d') => { // delete task
                if let Some((key, task)) = self.task_list.selected_task(&self.state) {
                    self.task_popup = Some(TaskPopup::Delete(key, task.name.clone()));
                }
            },
            /* Char('e') => {
                if let Some(selection) = self.task_list.selected_task {
                    self.taks = Some(TaskEditPopup::new(Some(selection)));
                }
            } */
            Up => self.task_list.shift(-1, false),
            Down => self.task_list.shift(1, false),
            Enter => {
                if let Some((selected_key, _)) = self.task_list.selected_task(&self.state) {
                    let res = self.state.task_mod(selected_key, |t| t.completed = !t.completed);
                    if let Err(err) = res {
                        self.report_error(err);
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
            "<h> ".blue().bold(),
            ", Quit: ".into(),
            "<q> ".blue().bold(),
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

        // render help list
        if self.help_box_shown {
            // create a centered rect of fixed vertical size that takes up 50% of the vertical area.
            let vertical_center = Layout::vertical([Constraint::Length(7)])
            .flex(layout::Flex::Center)
            .split(area);

            let popup_area = Layout::horizontal([Constraint::Percentage(50)])
                .flex(layout::Flex::Center)
                .split(vertical_center[0])[0];
            
            Clear.render(popup_area, buf); // clear background of popup area

            // create task popup block with rounded corners
            let block = Block::default()
                .title("Help Menu")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED);
            let text = vec![
                Line::from(vec![
                    Span::raw("Quit: "),
                    Span::styled("<q>", Style::new().blue().bold()),
                ]),
                Line::from(vec![
                    Span::raw("Help: "),
                    Span::styled("<h>", Style::new().blue().bold()),
                ]),
                Line::from(vec![
                    Span::raw("Create Task: "),
                    Span::styled("<c>", Style::new().blue().bold()),
                ]),
                Line::from(vec![
                    Span::raw("Delete Task: "),
                    Span::styled("<d>", Style::new().blue().bold()),
                ]),
                Line::from(vec![
                    Span::raw("Edit Task: "),
                    Span::styled("<e>", Style::new().blue().bold()),
                ]),
            ];
            // create paragraph containing current string state inside `block` & render
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(block)
                .render(popup_area, buf);
        }

        // popup rendering
        self.task_popup.as_ref().inspect(|t|t.render(area, buf));
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
        buffer_copy.set_style(*buffer_copy.area(), Style::reset());
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
        app.task_list.source_views_mod(&app.state, |s|s.extend(app.state.view_get_default())); // set the view key as is currently done in run()
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
        app.task_list.source_views_mod(&app.state, |s|s.push(app.state.view_get_default().unwrap()));
        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));

        assert_eq!(app.task_list.list_state.selected(), Some(0));

        app.handle_event(UserEvent(Event::Key(KeyCode::Down.into())));
        assert_eq!(app.task_list.list_state.selected(), Some(1));

        // test enter key
        app.handle_key_event(KeyCode::Enter.into());
        assert!(
            app.state
                .task_get(
                    app.state
                        .view_task_keys(app.state.view_get_default().unwrap())
                        .unwrap()[1]
                )
                .unwrap()
                .completed
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
        assert!(app.should_exit);

        let mut app = App::new(State::new().0);
        app.handle_key_event(KeyCode::Char('.').into());
        assert!(!app.should_exit);

        let mut app = App::new(State::new().0);
        app.handle_event(UserEvent(Event::FocusLost));
        assert!(!app.should_exit);

        // Test Edit
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.source_views_mod(&app.state, |s|s.push(app.state.view_get_default().unwrap()));

        /* app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        assert!(app.task_popup.is_none());
        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        assert!(app.task_popup.is_some());

        // Initial task name from popup is empty
        if let Some(task_popup) = &app.task_popup {
            if let TaskPopup::Create(name)
            assert!(task_popup.selection.is_some());
            assert!(!task_popup.should_close);
            assert_eq!(task_popup.name, "");
        } */

        /* // Cancel Editing
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        assert!(app.task_popup.is_some());
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('n').into())));
        assert!(app.task_popup.unwrap().should_close);

        // Confirm Editing
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        assert!(app.task_popup.is_some());
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('y').into())));
        assert!(!app.task_popup.unwrap().should_close);

        // Edit current task name
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('y').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('h').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('i').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Enter.into())));
        let task_keys = app
            .state
            .view_task_keys(app.state.view_get_default().unwrap())
            .unwrap();
        let updated_task_key = task_keys[0];
        let updated_task = app.state.task_get(updated_task_key).unwrap();
        assert_eq!(updated_task.name, "hi");

        // Press esc to cancel editing
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('y').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('h').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('i').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Esc.into())));
        assert!(app.task_popup.unwrap().should_close);

        // 'n' does not close popup
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('y').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('n').into())));
        assert!(!app.task_popup.unwrap().should_close);

        //
        let mut app = App::new(State::new().0);
        let state = init_test().0;
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('x').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('y').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('n').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Char('o').into())));
        app.handle_event(UserEvent(Event::Key(KeyCode::Enter.into())));
        let task_keys = app
            .state
            .view_task_keys(app.state.view_get_default().unwrap())
            .unwrap();
        let updated_task_key = task_keys[0];
        let updated_task = app.state.task_get(updated_task_key).unwrap();
        assert!(app.task_popup.is_some());
        assert_eq!(updated_task.name, "no");
        assert!(app.task_popup.unwrap().should_close); */

        Ok(())
    }
}
