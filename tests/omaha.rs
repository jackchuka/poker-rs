#![cfg(feature = "omaha")]

use poker_rs::cards::{Card, Rank, Suit};
use poker_rs::evaluator::Category;
use poker_rs::hand::Board;
use poker_rs::variants::omaha::{compare_omaha, evaluate_omaha, OmahaError, OmahaHoleCards};

fn hole(a: Card, b: Card, c: Card, d: Card) -> OmahaHoleCards {
    OmahaHoleCards::try_new(a, b, c, d).expect("valid hole cards")
}

#[test]
fn omaha_requires_two_hole_cards() {
    let board = Board::try_new(vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
        Card::new(Rank::Two, Suit::Diamonds),
    ])
    .unwrap();
    let a = hole(
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Three, Suit::Clubs),
        Card::new(Rank::Four, Suit::Diamonds),
    );
    let b = hole(
        Card::new(Rank::Two, Suit::Hearts),
        Card::new(Rank::Two, Suit::Spades),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Nine, Suit::Diamonds),
    );

    let ea = evaluate_omaha(&a, &board).unwrap();
    let eb = evaluate_omaha(&b, &board).unwrap();
    assert!(matches!(ea.category, Category::ThreeOfAKind));
    assert!(matches!(eb.category, Category::FourOfAKind));
    assert!(eb > ea);
    let ord = compare_omaha(&a, &b, &board).unwrap();
    assert!(ord.is_lt());
}

#[test]
fn omaha_rejects_short_board() {
    let board = Board::try_new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::Two, Suit::Clubs),
        Card::new(Rank::Three, Suit::Clubs),
        Card::new(Rank::Four, Suit::Clubs),
        Card::new(Rank::Five, Suit::Clubs),
    );
    let err = evaluate_omaha(&hole, &board).unwrap_err();
    assert!(matches!(err, OmahaError::BoardCount(4)));
}

#[test]
fn omaha_rejects_overlap() {
    let board = Board::try_new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Two, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Three, Suit::Clubs),
        Card::new(Rank::Four, Suit::Clubs),
        Card::new(Rank::Five, Suit::Clubs),
    );
    let err = evaluate_omaha(&hole, &board).unwrap_err();
    assert!(matches!(err, OmahaError::Overlap));
}
