use crate::tui::app::{AppState, InputAction, Scene};
use crate::tui::ui;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut AppState,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_key(app, key.code) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.agents_on_turn();
            last_tick = Instant::now();
        }
    }
    Ok(())
}

fn handle_key(app: &mut AppState, code: KeyCode) -> bool {
    let help_toggle = matches!(code, KeyCode::Char('?'));
    let history_toggle = matches!(code, KeyCode::Char('h') | KeyCode::Char('H'));
    if help_toggle {
        let _ = app.handle_input(InputAction::ToggleHelp);
        return false;
    }
    if history_toggle {
        let _ = app.handle_input(InputAction::ToggleHistory);
        return false;
    }
    if app.help_open() {
        if matches!(code, KeyCode::Esc) {
            let _ = app.handle_input(InputAction::ToggleHelp);
        }
        return false;
    }
    if app.history_open() {
        match code {
            KeyCode::Up => {
                let _ = app.handle_input(InputAction::HistoryUp);
            }
            KeyCode::Down => {
                let _ = app.handle_input(InputAction::HistoryDown);
            }
            KeyCode::Esc => {
                let _ = app.handle_input(InputAction::ToggleHistory);
            }
            _ => {}
        }
        return false;
    }
    if app.amount_entry_active() {
        match code {
            KeyCode::Esc => {
                let _ = app.handle_input(InputAction::AmountCancel);
            }
            KeyCode::Enter => {
                if app.handle_input(InputAction::AmountSubmit) {
                    app.agents_on_turn();
                }
            }
            KeyCode::Backspace => {
                let _ = app.handle_input(InputAction::AmountBackspace);
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let _ = app.handle_input(InputAction::AmountIncBb);
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                let _ = app.handle_input(InputAction::AmountDecBb);
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let _ = app.handle_input(InputAction::AmountDigit(c as u8 - b'0'));
            }
            _ => {}
        }
        return false;
    }

    match app.scene {
        Scene::Menu => match code {
            KeyCode::Up => {
                let _ = app.handle_input(InputAction::MenuPrev);
            }
            KeyCode::Down => {
                let _ = app.handle_input(InputAction::MenuNext);
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let _ = app.handle_input(InputAction::MenuInc);
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                let _ = app.handle_input(InputAction::MenuDec);
            }
            KeyCode::Enter => {
                let _ = app.handle_input(InputAction::MenuApply);
            }
            KeyCode::Esc => {
                let _ = app.handle_input(InputAction::MenuCancel);
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                let _ = app.handle_input(InputAction::ToggleMenu);
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => return true,
            _ => {}
        },
        Scene::Table => match code {
            KeyCode::Char('m') | KeyCode::Char('M') => {
                let _ = app.handle_input(InputAction::ToggleMenu);
            }
            KeyCode::Char(' ') => {
                let _ = app.handle_input(InputAction::NewHand);
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                let _ = app.handle_input(InputAction::AmountOpen);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                let _ = app.handle_input(InputAction::BotDifficultyNext);
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if app.handle_input(InputAction::Fold) {
                    app.agents_on_turn();
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if app.handle_input(InputAction::CheckCall) {
                    app.agents_on_turn();
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                if app.handle_input(InputAction::BetMin) {
                    app.agents_on_turn();
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if app.handle_input(InputAction::RaiseMin) {
                    app.agents_on_turn();
                }
            }
            KeyCode::Char(']') => {
                let _ = app.handle_input(InputAction::FocusNext);
            }
            KeyCode::Char('[') => {
                let _ = app.handle_input(InputAction::FocusPrev);
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let idx = (c as u8 - b'1') as usize;
                let _ = app.handle_input(InputAction::FocusSeat(idx));
            }
            _ => {}
        },
    }
    false
}
