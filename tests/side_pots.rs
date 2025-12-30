use poker_rs::cards::{Card, Rank, Suit};
use poker_rs::game::{Game, PlayerStatus, Street};
use poker_rs::hand::{Board, HoleCards};

fn mk_game(n: usize) -> Game {
    Game::new(n, 1000, 5, 10)
}

fn hole(a: Card, b: Card) -> HoleCards {
    HoleCards::try_new(a, b).expect("valid hole cards")
}

#[test]
fn side_pots_distribute_across_all_in_levels() {
    let mut g = mk_game(3);
    g.street = Street::Showdown;
    g.board = Board::new(vec![
        Card::new(Rank::Two, Suit::Clubs),
        Card::new(Rank::Three, Suit::Diamonds),
        Card::new(Rank::Four, Suit::Hearts),
        Card::new(Rank::Eight, Suit::Spades),
        Card::new(Rank::King, Suit::Clubs),
    ]);

    g.players[0].hole =
        Some(hole(Card::new(Rank::Queen, Suit::Spades), Card::new(Rank::Queen, Suit::Hearts)));
    g.players[1].hole =
        Some(hole(Card::new(Rank::Ace, Suit::Spades), Card::new(Rank::Ace, Suit::Hearts)));
    g.players[2].hole =
        Some(hole(Card::new(Rank::Seven, Suit::Clubs), Card::new(Rank::Six, Suit::Clubs)));

    g.players[0].status = PlayerStatus::AllIn;
    g.players[1].status = PlayerStatus::AllIn;
    g.players[2].status = PlayerStatus::AllIn;

    g.players[0].contributed = 100;
    g.players[1].contributed = 50;
    g.players[2].contributed = 200;
    g.pot = 350;

    g.players[0].stack = 0;
    g.players[1].stack = 0;
    g.players[2].stack = 0;

    g.finish_showdown();

    assert_eq!(g.players[1].stack, 150, "main pot should go to best hand");
    assert_eq!(g.players[0].stack, 100, "side pot should go to next best hand");
    assert_eq!(g.players[2].stack, 100, "single-eligible side pot goes to contributor");
}

#[test]
fn split_main_pot_and_single_side_pot() {
    let mut g = mk_game(3);
    g.street = Street::Showdown;
    g.board = Board::new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Two, Suit::Clubs),
    ]);

    g.players[0].hole =
        Some(hole(Card::new(Rank::Ten, Suit::Clubs), Card::new(Rank::Three, Suit::Diamonds)));
    g.players[1].hole =
        Some(hole(Card::new(Rank::Ten, Suit::Hearts), Card::new(Rank::Four, Suit::Spades)));
    g.players[2].hole =
        Some(hole(Card::new(Rank::Nine, Suit::Clubs), Card::new(Rank::Nine, Suit::Diamonds)));

    for p in &mut g.players {
        p.status = PlayerStatus::AllIn;
        p.stack = 0;
    }

    g.players[0].contributed = 50;
    g.players[1].contributed = 50;
    g.players[2].contributed = 200;
    g.pot = 300;

    g.finish_showdown();

    assert_eq!(g.players[0].stack, 75, "main pot split between tied winners");
    assert_eq!(g.players[1].stack, 75, "main pot split between tied winners");
    assert_eq!(g.players[2].stack, 150, "side pot goes to lone contributor");
}

#[test]
fn split_main_and_side_pots() {
    let mut g = mk_game(4);
    g.street = Street::Showdown;
    g.board = Board::new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Two, Suit::Clubs),
    ]);

    g.players[0].hole =
        Some(hole(Card::new(Rank::Ten, Suit::Clubs), Card::new(Rank::Three, Suit::Diamonds)));
    g.players[1].hole =
        Some(hole(Card::new(Rank::Ten, Suit::Hearts), Card::new(Rank::Four, Suit::Spades)));
    g.players[2].hole =
        Some(hole(Card::new(Rank::Nine, Suit::Clubs), Card::new(Rank::Nine, Suit::Diamonds)));
    g.players[3].hole =
        Some(hole(Card::new(Rank::Nine, Suit::Hearts), Card::new(Rank::Nine, Suit::Spades)));

    for p in &mut g.players {
        p.status = PlayerStatus::AllIn;
        p.stack = 0;
    }

    g.players[0].contributed = 50;
    g.players[1].contributed = 50;
    g.players[2].contributed = 100;
    g.players[3].contributed = 100;
    g.pot = 300;

    g.finish_showdown();

    assert_eq!(g.players[0].stack, 100, "main pot split between tied winners");
    assert_eq!(g.players[1].stack, 100, "main pot split between tied winners");
    assert_eq!(g.players[2].stack, 50, "side pot split between tied winners");
    assert_eq!(g.players[3].stack, 50, "side pot split between tied winners");
}

#[test]
fn odd_chip_split_uses_seat_order() {
    let mut g = mk_game(3);
    g.street = Street::Showdown;
    g.dealer = 0;
    g.board = Board::new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Two, Suit::Clubs),
    ]);

    g.players[0].hole =
        Some(hole(Card::new(Rank::Ten, Suit::Clubs), Card::new(Rank::Three, Suit::Diamonds)));
    g.players[1].hole =
        Some(hole(Card::new(Rank::Ten, Suit::Hearts), Card::new(Rank::Four, Suit::Spades)));
    g.players[2].hole =
        Some(hole(Card::new(Rank::Nine, Suit::Clubs), Card::new(Rank::Nine, Suit::Diamonds)));

    for p in &mut g.players {
        p.status = PlayerStatus::AllIn;
        p.stack = 0;
    }

    g.players[0].contributed = 1;
    g.players[1].contributed = 1;
    g.players[2].contributed = 2;
    g.pot = 4;

    g.finish_showdown();

    assert_eq!(g.players[0].stack, 1, "tie loser should receive smaller share");
    assert_eq!(g.players[1].stack, 2, "odd chip awarded by seat order");
    assert_eq!(g.players[2].stack, 1, "single-eligible side pot still awarded");
}
