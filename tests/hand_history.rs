use poker_rs::game::{Game, HandHistoryVerb, Street};

#[test]
fn history_records_blinds_and_actions() {
    let mut game = Game::new(2, 1000, 5, 10);
    game.new_hand();

    let history = game.history_recent(10);
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].verb, HandHistoryVerb::SmallBlind);
    assert_eq!(history[0].amount, Some(5));
    assert_eq!(history[0].street, Street::Preflop);
    assert_eq!(history[1].verb, HandHistoryVerb::BigBlind);
    assert_eq!(history[1].amount, Some(10));
    assert_eq!(history[1].street, Street::Preflop);

    game.action_check_call().unwrap();
    let recent = game.history_recent(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].verb, HandHistoryVerb::Call);
    assert_eq!(recent[0].amount, Some(5));
    assert_eq!(recent[0].street, Street::Preflop);
}

#[test]
fn history_offset_pages_from_the_end() {
    let mut game = Game::new(2, 1000, 1, 2);
    game.new_hand();
    for _ in 0..5 {
        game.action_check_call().unwrap();
    }
    let total = game.history_len();
    assert!(total >= 7);

    let window = game.history_recent_offset(3, 0);
    assert_eq!(window.len(), 3);
    let older = game.history_recent_offset(3, 2);
    assert_eq!(older.len(), 3);
    assert_ne!(window[0], older[0]);
}
