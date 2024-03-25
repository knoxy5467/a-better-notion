use std::io::{self, stdout, Stdout};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn wrap_terminal(func: impl Fn(&mut Tui) -> io::Result<()>) -> io::Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let result = func(&mut terminal);
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_wrap() {
        #[allow(unused_must_use)]
        let _ = wrap_terminal(|_| Ok(()));
    }
}
