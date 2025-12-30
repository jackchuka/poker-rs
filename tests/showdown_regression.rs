use poker_rs::cards::{Card, Rank::*, Suit::*};
use poker_rs::evaluator::{compare_holdem, evaluate_holdem, Category};
use poker_rs::hand::{Board, HoleCards};

fn hole(a: Card, b: Card) -> HoleCards {
    HoleCards::try_new(a, b).expect("valid hole cards")
}

// Scenario inspired by the screenshot: board has a Jack; P1 has Jx; P5 has 76.
// P1 should beat P5 with Pair(J) vs Pair(6).
#[test]
fn pair_beats_lower_pair() {
    let board = Board::new(vec![
        Card::new(Ace, Clubs),
        Card::new(Nine, Diamonds),
        Card::new(Four, Spades),
        Card::new(Two, Hearts),
        Card::new(Six, Clubs),
    ]);
    let a = hole(Card::new(King, Spades), Card::new(King, Hearts));
    let b = hole(Card::new(Queen, Clubs), Card::new(Queen, Hearts));
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "Higher pair should beat lower pair");
}

#[test]
fn pair_kicker_breaks_ties() {
    let board = Board::new(vec![
        Card::new(King, Clubs),
        Card::new(Nine, Diamonds),
        Card::new(Four, Spades),
        Card::new(Two, Hearts),
        Card::new(Six, Clubs),
    ]);
    let a = hole(Card::new(King, Spades), Card::new(Ace, Diamonds));
    let b = hole(Card::new(King, Hearts), Card::new(Queen, Diamonds));
    // both pairs of Kings; A kicker beats Q kicker
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "Pair with higher kicker should win");
}

#[test]
fn two_pair_ordering_high_then_low_then_kicker() {
    let board = Board::new(vec![
        Card::new(King, Clubs),
        Card::new(Nine, Diamonds),
        Card::new(Four, Spades),
        Card::new(Two, Hearts),
        Card::new(Ace, Clubs),
    ]);
    let a = hole(Card::new(King, Spades), Card::new(Nine, Clubs));
    let b = hole(Card::new(King, Hearts), Card::new(Two, Diamonds));
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "K9 two pair should beat K2 two pair");
}

#[test]
fn trips_order_by_trip_rank() {
    let board = Board::new(vec![
        Card::new(Queen, Clubs),
        Card::new(Nine, Diamonds),
        Card::new(Four, Spades),
        Card::new(Two, Hearts),
        Card::new(Six, Clubs),
    ]);
    let a = hole(Card::new(Queen, Spades), Card::new(Queen, Diamonds));
    let b = hole(Card::new(Nine, Clubs), Card::new(Nine, Hearts));
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "Trips Q should beat Trips 9");
}

#[test]
fn straight_top_card_and_wheel() {
    let board = Board::new(vec![
        Card::new(Five, Clubs),
        Card::new(Four, Diamonds),
        Card::new(Three, Spades),
        Card::new(Two, Hearts),
        Card::new(King, Clubs),
    ]);
    let a = hole(Card::new(Ace, Diamonds), Card::new(Nine, Clubs)); // A-5 straight
    let b = hole(Card::new(Six, Diamonds), Card::new(Nine, Hearts)); // 6-high straight
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_lt(), "6-high straight should beat wheel A-5");
}

#[test]
fn flush_order_by_kickers() {
    let board = Board::new(vec![
        Card::new(Ace, Clubs),
        Card::new(Nine, Clubs),
        Card::new(Four, Clubs),
        Card::new(Two, Clubs),
        Card::new(Six, Diamonds),
    ]);
    let a = hole(Card::new(King, Clubs), Card::new(Queen, Diamonds));
    let b = hole(Card::new(Queen, Clubs), Card::new(Jack, Diamonds));
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "Flush with higher second card should win");
}

#[test]
fn full_house_ordering_trips_then_pair() {
    let board = Board::new(vec![
        Card::new(King, Clubs),
        Card::new(King, Diamonds),
        Card::new(Four, Spades),
        Card::new(Four, Hearts),
        Card::new(Two, Clubs),
    ]);
    let a = hole(Card::new(King, Spades), Card::new(Ace, Diamonds)); // KKK44
    let b = hole(Card::new(Four, Diamonds), Card::new(Ace, Hearts)); // 444KK
    let ea = evaluate_holdem(&a, &board).unwrap();
    let eb = evaluate_holdem(&b, &board).unwrap();
    assert!(matches!(ea.category, Category::FullHouse));
    assert!(matches!(eb.category, Category::FullHouse));
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "Full House with higher trips should win");
}

#[test]
fn quads_on_board_kicker_decides() {
    let board = Board::new(vec![
        Card::new(Nine, Clubs),
        Card::new(Nine, Diamonds),
        Card::new(Nine, Hearts),
        Card::new(Nine, Spades),
        Card::new(King, Clubs),
    ]);
    let a = hole(Card::new(Ace, Diamonds), Card::new(Two, Diamonds)); // kicker A
    let b = hole(Card::new(Queen, Diamonds), Card::new(Three, Diamonds)); // kicker Q
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_gt(), "With quads on board, higher kicker in hand should win");
}

#[test]
fn straight_flush_ordering() {
    let board = Board::new(vec![
        Card::new(Nine, Clubs),
        Card::new(Eight, Clubs),
        Card::new(Seven, Clubs),
        Card::new(Six, Clubs),
        Card::new(Two, Diamonds),
    ]);
    let a = hole(Card::new(Five, Clubs), Card::new(Ace, Diamonds)); // 9-8-7-6-5 straight flush
    let b = hole(Card::new(Ten, Clubs), Card::new(Ace, Hearts)); // T-9-8-7-6 straight flush
    let ord = compare_holdem(&a, &b, &board).unwrap();
    assert!(ord.is_lt(), "Higher straight flush should win");
}
