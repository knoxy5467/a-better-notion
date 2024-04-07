//! Terminal module
//! call enable() and restore() for real terminals
//! call create<W>(writer: W) to create the crossterm backend

use std::io::{self, stdout};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui<B> = Terminal<B>;

/// Enter alternate screen (required to initialize terminal)
#[coverage(off)]
pub fn enable() -> io::Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()
}

/// Leave alternate screen (cleanup crossterm)
#[coverage(off)]
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()
}

/* #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_wrap() {
        if std::env::var("TERM").is_ok() { return; } // This test does not work on CI
        // disable if running on github actions
        if std::env::var("GITHUB_ACTIONS").is_ok() { return; }

        let _ = enable().unwrap();
        let _ = restore().unwrap();
    }
} */
