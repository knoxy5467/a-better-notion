use std::io::{self, stdout, Stdout};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    #[test]
    fn test_init() {
        // Call the init function and assert that it returns a Tui instance
        let result = init();
        assert!(result.is_ok());
        assert!(result.unwrap().type_id() == std::any::TypeId::of::<Tui>());
    }

    #[test]
    fn test_restore() {
        // Call the restore function and assert that it returns Ok(())
        let result = restore();
        assert!(result.is_ok());
    }
}
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
