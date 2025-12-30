use poker_rs::cards::{Card, Rank, Suit};
use poker_rs::evaluator::{evaluate_five, Category};

#[test]
fn category_straight_flush() {
    let sf = [
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Ten, Suit::Spades),
    ];
    let e = evaluate_five(&sf);
    assert!(matches!(e.category, Category::StraightFlush));
}

#[test]
fn category_four_of_a_kind() {
    let xs = [
        Card::new(Rank::Nine, Suit::Clubs),
        Card::new(Rank::Nine, Suit::Diamonds),
        Card::new(Rank::Nine, Suit::Hearts),
        Card::new(Rank::Nine, Suit::Spades),
        Card::new(Rank::Ace, Suit::Clubs),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::FourOfAKind));
}

#[test]
fn category_full_house() {
    let xs = [
        Card::new(Rank::Three, Suit::Clubs),
        Card::new(Rank::Three, Suit::Diamonds),
        Card::new(Rank::Three, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Jack, Suit::Clubs),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::FullHouse));
}

#[test]
fn category_flush() {
    let xs = [
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Eight, Suit::Hearts),
        Card::new(Rank::Six, Suit::Hearts),
        Card::new(Rank::Three, Suit::Hearts),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::Flush));
}

#[test]
fn category_straight() {
    let xs = [
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Five, Suit::Clubs),
        Card::new(Rank::Four, Suit::Diamonds),
        Card::new(Rank::Three, Suit::Hearts),
        Card::new(Rank::Two, Suit::Spades),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::Straight));
}

#[test]
fn category_three_of_a_kind() {
    let xs = [
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Ten, Suit::Spades),
        Card::new(Rank::Two, Suit::Clubs),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::ThreeOfAKind));
}

#[test]
fn category_two_pair() {
    let xs = [
        Card::new(Rank::Jack, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Diamonds),
        Card::new(Rank::Nine, Suit::Clubs),
        Card::new(Rank::Nine, Suit::Hearts),
        Card::new(Rank::Two, Suit::Spades),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::TwoPair));
}

#[test]
fn category_pair() {
    let xs = [
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Ace, Suit::Diamonds),
        Card::new(Rank::Ten, Suit::Spades),
        Card::new(Rank::Nine, Suit::Clubs),
        Card::new(Rank::Two, Suit::Diamonds),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::Pair));
}

#[test]
fn category_high_card() {
    let xs = [
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Seven, Suit::Spades),
        Card::new(Rank::Five, Suit::Clubs),
        Card::new(Rank::Two, Suit::Diamonds),
    ];
    let e = evaluate_five(&xs);
    assert!(matches!(e.category, Category::HighCard));
}
