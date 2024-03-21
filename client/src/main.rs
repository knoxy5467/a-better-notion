//! Client

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

mod mid;
mod term;

fn main() -> io::Result<()> {
    let mut terminal = term::init()?;
    let app_result = App::default().run(&mut terminal);
    term::restore()?;
    app_result
}

/// UI App State
#[derive(Debug, Default)]
pub struct App {
    counter: i16,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut term::Tui) -> io::Result<()> {
        self.counter = 9001;
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.increment_counter(),
            KeyCode::Right => self.decrement_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn decrement_counter(&mut self) {
        self.counter += 1;
    }

    fn increment_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Task Management ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
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

        let counter_text = Text::from(vec![
            "Usage:".into(),
            "Press <Button> To Do <X>".into(),
            "Press <Other Button> To Do <Y>".into(),
            "↑ ↑ ↓ ↓ ← → ← → B A".into(),
            Line::from(vec!["Level: ".into(), self.counter.to_string().yellow()]),
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* #[test]
    fn render() {
        let app = App::default();
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
    } */

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.exit, true);

        Ok(())
    }
}
