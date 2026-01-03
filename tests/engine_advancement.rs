use poker_rs::game::{Game, Street};

fn advance_with_checks(game: &mut Game, street: Street) {
    let max_steps = game.players().len() + 3;
    for _ in 0..max_steps {
        if game.street() != street {
            return;
        }
        game.action_check_call().unwrap();
    }
    panic!("street did not advance from {street:?}");
}

#[test]
fn auto_showdown_when_all_players_all_in() {
    let mut game = Game::new(3, 20, 5, 10);
    game.new_hand();

    game.action_raise_to(20).unwrap();
    game.action_check_call().unwrap();
    game.action_check_call().unwrap();

    assert!(matches!(game.street(), Street::Showdown));
    assert_eq!(game.board().len(), 5);
}

#[test]
fn check_down_advances_to_showdown() {
    let mut game = Game::new(3, 100, 5, 10);
    game.new_hand();

    advance_with_checks(&mut game, Street::Preflop);
    advance_with_checks(&mut game, Street::Flop);
    advance_with_checks(&mut game, Street::Turn);
    advance_with_checks(&mut game, Street::River);

    assert!(matches!(game.street(), Street::Showdown));
    assert_eq!(game.board().len(), 5);
}

#[test]
fn postflop_bet_and_calls_advance() {
    let mut game = Game::new(3, 100, 5, 10);
    game.new_hand();

    advance_with_checks(&mut game, Street::Preflop);
    assert!(matches!(game.street(), Street::Flop));

    game.action_bet_min().unwrap();
    advance_with_checks(&mut game, Street::Flop);

    assert!(matches!(game.street(), Street::Turn));
}
