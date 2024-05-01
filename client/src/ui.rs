use std::io;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::{channel::mpsc::Receiver, Stream, StreamExt};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use crate::{mid::{MidEvent, State, StateEvent}, term};

mod task_list;

const BACKGROUND: Color = Color::Reset;
const TEXT_COLOR: Color = Color::White;
const GREYED_OUT_TEXT_COLOR: Color = Color::Gray;
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
            UIEvent::UserEvent(event) => self.handle_term_event(event),
            UIEvent::StateEvent(state_event) => match state_event {
                StateEvent::TasksUpdate => true,
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
    // handle crossterm events, return boolean value to determine whether screen should be re-rendered or not given the event
    fn handle_term_event(&mut self, event: Event) -> bool {
        use KeyCode::*;

        // pass event to task list to check if it handles the event, if not, handle it below
        if self.task_list.handle_term_event(&mut self.state, &event) {
            return true;
        }
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if let Char('h') = key_event.code {} else { self.help_box_shown = false; }
                match key_event.code {
                    Esc => if self.help_box_shown { self.help_box_shown = false; }
                    Char('q') => self.should_exit = true,
                    Char('h') => self.help_box_shown = !self.help_box_shown,
                    _ => return false,
                }
            }
            Event::Resize(_, _) => (),
            _ => (),
        }
        
        true // assume we should re-render if we didn't explicitly return false somewhere above.
    }
}

pub fn report_error(error: impl std::error::Error) {
    tracing::error!("{error}");
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

    #[tokio::test]
    async fn render_test() -> color_eyre::Result<()> {
        // test default state
        let (_, mut term) = create_render_test(State::new().0, 55, 5);

        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│         No Tasks, Have you Selected a View?         │",
            "│                                                     │",
            "│                                                     │",
            "╰───── Select: <Up>/<Down> Help: <h> , Quit: <q> es: 1╯",
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
            "╰───── Select: <Up>/<Down> Help: <h> , Quit: <q> es: 2╯",
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
            "╰───── Select: <Up>/<Down> Help: <h> , Quit: <q> es: 3╯",
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
            "│             ╭Edit Task────────────────╮             │",
            "│             │Finish ABNhi             │             │",
            "│             ╰─────────────────────────╯             │",
            "│                                                     │",
            "╰───── Select: <Up>/<Down> Help: <h> , Quit: <q> es: 8╯",
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
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "╰───── Select: <Up>/<Down> Help: <h> , Quit: <q> s: 11╯",
        ]);
        term.backend().assert_buffer(&expected);

        // test task deletion
        app.step(&mut term, Event::Key(KeyCode::Char('d').into()))?;
        app.step(&mut term, Event::Key(KeyCode::Char('h').into()))?;
        app.step(&mut term, Event::Key(KeyCode::Char('!').into()))?;
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ Finish ABN                                       │",
            "│  ☐ hi       ╭Delete Task──────────────╮             │",
            "│             │You sure man? [Y/N]      │             │",
            "│             ╰─────────────────────────╯             │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> Updates: 12╯",
        ]);
        term.backend().assert_buffer(&expected);

        app.step(&mut term, Event::Key(KeyCode::Char('n').into()))?;
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ Finish ABN                                       │",
            "│  ☐ hi                                               │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> Updates: 13╯",
        ]);
        term.backend().assert_buffer(&expected);

        app.step(&mut term, Event::Key(KeyCode::Char('d').into()))?;
        app.step(&mut term, Event::Key(KeyCode::Char('y').into()))?;
        reset_buffer_style(&mut term);
        let expected = Buffer::with_lines(vec![
            "╭────────────────── Task Management ──────────────────╮",
            "│  ✓ Eat Lunch                                        │",
            "│> ☐ hi                                               │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "│                                                     │",
            "╰────────── Select: <Up>/<Down>, Quit: <Q> Updates: 15╯",
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
        /* app.handle_key_event(KeyCode::Enter.into());
        assert!(
            app.state
                .task_get(
                    app.task_list
                )
                .unwrap()
                .completed
        ); // second task in example view is marked as completed, so the Enter key should uncomplete it */

        // test up and down in regular state
        let mut app = App::new(State::new().0);
        app.handle_event(UserEvent(Event::Key(KeyCode::Up.into())));
        assert_eq!(app.task_list.list_state.selected(), None);
        app.handle_event(UserEvent(Event::Key(KeyCode::Down.into())));
        assert_eq!(app.task_list.list_state.selected(), None);
        /* app.handle_key_event(KeyCode::Enter.into());

        let mut app = App::new(State::new().0);
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.should_exit);

        let mut app = App::new(State::new().0);
        app.handle_key_event(KeyCode::Char('.').into());
        assert!(!app.should_exit); */

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

        // test delete task
        let mut app = App::new(State::default());

        let state = init_test();
        app.state = state;
        app.task_list.current_view = Some(app.state.view_get_default().unwrap());

        app.handle_event(Event::Key(KeyCode::Up.into()));
        app.handle_event(Event::Key(KeyCode::Char('d').into()));

        if let Some(task_delete_popup) = &app.task_delete_popup {
            assert!(!task_delete_popup.should_close);
        } else { assert!(false) }

        app.handle_event(Event::Key(KeyCode::Char('e').into()));
        assert!(app.task_create_popup.is_none());

        app.handle_event(Event::Key(KeyCode::Char('n').into()));
        if let Some(task_delete_popup) = &app.task_delete_popup {
            assert!(task_delete_popup.should_close);
        } else { assert!(false) }

        app.handle_event(Event::Key(KeyCode::Char('d').into()));
 
        let selected = app.task_list.selected_task.unwrap();
        app.handle_event(Event::Key(KeyCode::Char('y').into()));
        if let Some(task_delete_popup) = &app.task_delete_popup {
            assert!(task_delete_popup.should_close);
        } else { assert!(false) }
        assert!(app.state.task_get(selected).is_none());

        Ok(())
    }
}
