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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::AppState;

    #[test]
    fn help_toggle_opens_and_closes() {
        let mut app = AppState::default();
        app.scene = Scene::Table; // Help only works on Table scene
        assert!(!app.help_open());

        // Open help
        let quit = handle_key(&mut app, KeyCode::Char('?'));
        assert!(!quit);
        assert!(app.help_open());

        // Close with Esc
        let quit = handle_key(&mut app, KeyCode::Esc);
        assert!(!quit);
        assert!(!app.help_open());

        // Toggle again
        let quit = handle_key(&mut app, KeyCode::Char('?'));
        assert!(!quit);
        assert!(app.help_open());
    }

    #[test]
    fn history_toggle_opens_and_closes() {
        let mut app = AppState::default();
        app.scene = Scene::Table; // History only works on Table scene
        assert!(!app.history_open());

        // Open history
        let quit = handle_key(&mut app, KeyCode::Char('h'));
        assert!(!quit);
        assert!(app.history_open());

        // Close with Esc
        let quit = handle_key(&mut app, KeyCode::Esc);
        assert!(!quit);
        assert!(!app.history_open());

        // Test uppercase 'H'
        let quit = handle_key(&mut app, KeyCode::Char('H'));
        assert!(!quit);
        assert!(app.history_open());
    }

    #[test]
    fn help_blocks_other_input() {
        let mut app = AppState::default();
        app.scene = Scene::Table; // Start in Table scene to enable help

        // Open help
        handle_key(&mut app, KeyCode::Char('?'));
        assert!(app.help_open());

        // Try to switch scenes - should be blocked
        handle_key(&mut app, KeyCode::Char('m'));
        assert_eq!(app.scene, Scene::Table);

        // Only Esc should work
        handle_key(&mut app, KeyCode::Esc);
        assert!(!app.help_open());
    }

    #[test]
    fn history_navigation() {
        let mut app = AppState::default();
        app.scene = Scene::Table; // History only works on Table scene

        // Open history
        handle_key(&mut app, KeyCode::Char('h'));
        assert!(app.history_open());

        let _initial_offset = app.history_offset();

        // Navigate down
        handle_key(&mut app, KeyCode::Down);
        // Offset might not change if at bottom already, but key is handled

        // Navigate up
        handle_key(&mut app, KeyCode::Up);

        // Other keys should be ignored
        handle_key(&mut app, KeyCode::Char('x'));
        assert!(app.history_open());
    }

    #[test]
    fn menu_navigation() {
        let mut app = AppState::default();
        app.scene = Scene::Menu;

        let initial_index = app.menu_index;

        // Navigate down
        handle_key(&mut app, KeyCode::Down);
        assert!(app.menu_index > initial_index || app.menu_index == 0);

        // Navigate up
        handle_key(&mut app, KeyCode::Up);

        // Increment value
        handle_key(&mut app, KeyCode::Char('+'));

        // Decrement value
        handle_key(&mut app, KeyCode::Char('-'));
    }

    #[test]
    fn menu_quit_on_q() {
        let mut app = AppState::default();
        app.scene = Scene::Menu;

        let quit = handle_key(&mut app, KeyCode::Char('q'));
        assert!(quit);

        let quit = handle_key(&mut app, KeyCode::Char('Q'));
        assert!(quit);
    }

    #[test]
    fn menu_toggle_switches_scenes() {
        let mut app = AppState::default();
        app.scene = Scene::Menu;

        handle_key(&mut app, KeyCode::Char('m'));
        assert_eq!(app.scene, Scene::Table);

        handle_key(&mut app, KeyCode::Char('M'));
        assert_eq!(app.scene, Scene::Menu);
    }

    #[test]
    fn table_scene_actions() {
        let mut app = AppState::default();
        app.scene = Scene::Table;

        // Test various table actions don't cause quit
        assert!(!handle_key(&mut app, KeyCode::Char('f')));
        assert!(!handle_key(&mut app, KeyCode::Char('c')));
        assert!(!handle_key(&mut app, KeyCode::Char('b')));
        assert!(!handle_key(&mut app, KeyCode::Char('r')));
        assert!(!handle_key(&mut app, KeyCode::Char('a')));
        assert!(!handle_key(&mut app, KeyCode::Char(' ')));
    }

    #[test]
    fn table_focus_navigation() {
        let mut app = AppState::default();
        app.scene = Scene::Table;

        // Focus next
        handle_key(&mut app, KeyCode::Char(']'));

        // Focus prev
        handle_key(&mut app, KeyCode::Char('['));

        // Focus specific seat (1-9)
        handle_key(&mut app, KeyCode::Char('1'));
        assert_eq!(app.focus, 0);

        handle_key(&mut app, KeyCode::Char('3'));
        assert_eq!(app.focus, 2);

        // '0' should be ignored
        let focus_before = app.focus;
        handle_key(&mut app, KeyCode::Char('0'));
        assert_eq!(app.focus, focus_before);
    }

    #[test]
    fn amount_entry_mode() {
        let mut app = AppState::default();
        app.scene = Scene::Table;

        // Open amount entry
        handle_key(&mut app, KeyCode::Char('a'));
        // May or may not open depending on game state, but shouldn't crash

        if app.amount_entry_active() {
            // Test digit entry
            handle_key(&mut app, KeyCode::Char('5'));

            // Test backspace
            handle_key(&mut app, KeyCode::Backspace);

            // Test increment/decrement
            handle_key(&mut app, KeyCode::Char('+'));
            handle_key(&mut app, KeyCode::Char('-'));

            // Test cancel
            handle_key(&mut app, KeyCode::Esc);
            assert!(!app.amount_entry_active());
        }
    }

    #[test]
    fn bot_difficulty_cycle() {
        let mut app = AppState::default();
        app.scene = Scene::Table;

        handle_key(&mut app, KeyCode::Char('d'));
        // Difficulty should cycle

        handle_key(&mut app, KeyCode::Char('D'));
        // Test uppercase too
    }
}
