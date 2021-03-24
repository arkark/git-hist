use anyhow::Result;
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::CrosstermBackend, Frame, Terminal as TuiTerminal};

pub fn initialize() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
    Ok(())
}

pub fn terminate() -> Result<()> {
    execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub struct Terminal {
    terminal: TuiTerminal<CrosstermBackend<io::Stdout>>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = TuiTerminal::new(backend)?;
        terminal.clear()?;
        Ok(Terminal { terminal })
    }

    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame<'_, CrosstermBackend<io::Stdout>>),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    pub fn height(&self) -> usize {
        self.terminal.size().unwrap().height.into()
    }
}
