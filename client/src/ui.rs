use std::io;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::{channel::mpsc::Receiver, Stream, StreamExt};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use crate::{
    mid::{MidEvent, State, StateEvent},
    term,
};

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
        self.task_list
            .source_views_mod(&self.state, |s| s.extend(self.state.view_get_default()));
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
                }
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
                if let Char('h') = key_event.code {
                } else {
                    self.help_box_shown = false;
                }
                match key_event.code {
                    Esc => {
                        if self.help_box_shown {
                            self.help_box_shown = false;
                        }
                    }
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
        app.task_list
            .source_views_mod(&app.state, |s| s.extend(app.state.view_get_default())); // set the view key as is currently done in run()
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
        Ok(())
    }
}
