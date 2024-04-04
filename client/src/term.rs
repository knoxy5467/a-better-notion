//! Terminal module
//! call enable() and restore() for real terminals
//! call create<W>(writer: W) to create the crossterm backend

use std::io::{self, Write};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui<W> = Terminal<CrosstermBackend<W>>;

/// Enter alternate screen (required to initialize terminal)
pub fn enable<W: Write>(mut writer: W) -> io::Result<()> {
    execute!(writer, EnterAlternateScreen)?;
    enable_raw_mode()
}

/// create the terminal object generic on writer used by ratatui
pub fn create<W: Write>(writer: W) -> io::Result<Tui<W>> {
    Terminal::new(CrosstermBackend::new(writer))
}

/// Leave alternate screen (cleanup crossterm)
pub fn restore<W: Write>(mut writer: W) -> io::Result<()> {
    execute!(writer, LeaveAlternateScreen)?;
    disable_raw_mode()
}

#[cfg(test)]
mod tests {
    use iobuffer::IoBuffer;
    use super::*;

    #[test]
    fn terminal_wrap() {
        let out = IoBuffer::new();
        let _ = enable(out.clone()).unwrap();
        let _ = create(out.clone()).unwrap();
        let _ = restore(out).unwrap();
    }
}
