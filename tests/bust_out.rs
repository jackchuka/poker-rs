use poker_rs::game::{Game, PlayerStatus};

#[test]
fn zero_stack_players_sit_out_next_hand() {
    let mut game = Game::new(3, 100, 5, 10);
    game.players[1].stack = 0;

    game.new_hand();

    let busted = &game.players[1];
    assert!(matches!(busted.status, PlayerStatus::Folded));
    assert!(busted.hole.is_none());
    assert_eq!(busted.bet, 0);
    assert_eq!(busted.contributed, 0);
    assert_ne!(game.current, 1);
}
