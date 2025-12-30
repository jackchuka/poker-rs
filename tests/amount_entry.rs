use poker_rs::game::{ActionError, Game, Street};

fn mk_game(n: usize) -> Game {
    Game::new(n, 1000, 5, 10)
}

#[test]
fn raise_to_requires_min_raise() {
    let mut g = mk_game(3);
    g.new_hand();
    let cur = g.current;
    let target = g.current_bet + g.min_raise - 1;
    let err = g.action_raise_to(target).unwrap_err();
    assert!(matches!(err, ActionError::AmountTooSmall { .. }));

    assert_eq!(g.current, cur);
    assert!(g.players[cur].last_action.is_none());
}

#[test]
fn raise_to_min_works() {
    let mut g = mk_game(3);
    g.new_hand();
    let cur = g.current;
    let target = g.current_bet + g.min_raise;
    g.action_raise_to(target).unwrap();

    assert_ne!(g.current, cur);
    assert!(matches!(g.players[cur].last_action.as_deref(), Some(s) if s.starts_with("Raise to")));
}

#[test]
fn bet_requires_no_current_bet_and_min_amount() {
    let mut g = mk_game(3);
    g.new_hand();
    while g.street == Street::Preflop {
        g.action_check_call().unwrap();
    }
    let cur = g.current;

    let err = g.action_bet(g.big_blind - 1).unwrap_err();
    assert!(matches!(err, ActionError::AmountTooSmall { .. }));
    assert_eq!(g.current, cur);
    assert!(g.players[cur].last_action.is_none());

    g.action_bet(g.big_blind).unwrap();
    assert_ne!(g.current, cur);
    assert!(matches!(g.players[cur].last_action.as_deref(), Some(s) if s.starts_with("Bet")));
}
