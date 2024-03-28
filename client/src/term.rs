use std::io::{self, stdout, Write};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui<W> = Terminal<CrosstermBackend<W>>;

pub fn init<W: Write>(writer: W) -> io::Result<Tui<W>> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(writer))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_wrap() {
        let mut out = Vec::<u8>::new();
        let _ = init::<&mut Vec<u8>>(&mut out);

        let _ = restore();
    }
}
