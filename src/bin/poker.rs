use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use poker_rs::tui::{app::AppState, controller};
use ratatui::prelude::*;
use std::io::{self, IsTerminal, Stdout};
use std::time::Duration;

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> io::Result<()> {
    if !io::stdout().is_terminal() {
        println!(
            "poker-rs TUI requires a real terminal (TTY).\nRun in Terminal and press q to quit. Version: {}",
            poker_rs::VERSION
        );
        return Ok(());
    }
    let mut terminal = setup_terminal()?;
    let tick_rate = Duration::from_millis(250);
    let mut app = AppState::default();

    let res = controller::run(&mut terminal, &mut app, tick_rate);

    // Always attempt to restore terminal
    restore_terminal(terminal)?;
    res
}
