use color_eyre::eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend, crossterm};

pub type Tui = Terminal<CrosstermBackend<std::io::Stderr>>;

pub fn init() -> Result<Tui> {
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
