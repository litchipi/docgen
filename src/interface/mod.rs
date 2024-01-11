use crossterm::ExecutableCommand;
use crossterm::event::Event;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub mod ask;
mod select_list;

pub use select_list::select_from_list;

use crate::errors::Errcode;

pub type Tuiterm = Terminal<CrosstermBackend<std::io::Stdout>>;

fn enter_tui_screen() -> Result<Tuiterm, Errcode> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(std::io::stdout()))?)
}

fn quit_tui_screen() -> Result<(), Errcode> {
    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn get_keyboard_events() -> Result<Option<Event>, Errcode> {
    if crossterm::event::poll(std::time::Duration::from_millis(50))? {
        let event = crossterm::event::read()?;
        Ok(Some(event))
    } else {
        Ok(None)
    }
}
