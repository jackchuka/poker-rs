use poker_rs::tui::app::{AppState, InputAction, Scene};

fn setup_table_app() -> AppState {
    let mut app = AppState::default();
    app.apply_menu();
    app
}

#[test]
fn menu_navigation_and_apply() {
    let mut app = AppState::default();
    assert!(matches!(app.scene, Scene::Menu));
    let start = app.menu_index;
    let _ = app.handle_input(InputAction::MenuNext);
    assert_ne!(app.menu_index, start);
    let _ = app.handle_input(InputAction::MenuPrev);
    assert_eq!(app.menu_index, start);
    let _ = app.handle_input(InputAction::MenuApply);
    assert!(matches!(app.scene, Scene::Table));
}

#[test]
fn help_and_history_toggle() {
    let mut app = setup_table_app();
    let _ = app.handle_input(InputAction::ToggleHelp);
    assert!(app.help_open());
    let _ = app.handle_input(InputAction::ToggleHistory);
    assert!(!app.help_open());
    assert!(app.history_open());
    let _ = app.handle_input(InputAction::ToggleHistory);
    assert!(!app.history_open());
}

#[test]
fn amount_entry_edit_and_cancel() {
    let mut app = setup_table_app();
    let _ = app.handle_input(InputAction::NewHand);
    let current = app.game.current;
    let _ = app.handle_input(InputAction::FocusSeat(current));

    let expected = if app.game.current_bet == 0 {
        app.game.big_blind.max(1).to_string()
    } else {
        (app.game.current_bet + app.game.min_raise).to_string()
    };

    assert!(app.handle_input(InputAction::AmountOpen));
    assert!(app.amount_entry_active());
    assert_eq!(app.amount_entry_text(), Some(expected.as_str()));

    let _ = app.handle_input(InputAction::AmountDigit(5));
    let appended = format!("{expected}5");
    assert_eq!(app.amount_entry_text(), Some(appended.as_str()));

    let _ = app.handle_input(InputAction::AmountBackspace);
    assert_eq!(app.amount_entry_text(), Some(expected.as_str()));

    let _ = app.handle_input(InputAction::AmountCancel);
    assert!(!app.amount_entry_active());
}

#[test]
fn focus_wraps_across_seats() {
    let mut app = setup_table_app();
    let n = app.game.players.len();
    assert!(n >= 2);
    app.focus = n - 1;
    let _ = app.handle_input(InputAction::FocusNext);
    assert_eq!(app.focus, 0);
    let _ = app.handle_input(InputAction::FocusPrev);
    assert_eq!(app.focus, n - 1);
}
