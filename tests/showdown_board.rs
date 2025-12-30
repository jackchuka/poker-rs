use poker_rs::game::{Game, PlayerStatus, Street};
use poker_rs::hand::Board;

#[test]
fn showdown_deals_remaining_board_cards() {
    let mut game = Game::new(3, 100, 5, 10);
    game.new_hand();

    for p in &mut game.players {
        p.status = PlayerStatus::Folded;
    }
    game.players[0].status = PlayerStatus::AllIn;
    game.players[1].status = PlayerStatus::AllIn;
    game.players[0].contributed = 50;
    game.players[1].contributed = 50;
    game.pot = 100;
    game.board = Board::new(Vec::new());
    game.street = Street::Showdown;

    game.finish_showdown();

    assert_eq!(game.board.len(), 5);
    assert!(game.pot == 0);
    assert!(!game.winners.is_empty());
}
