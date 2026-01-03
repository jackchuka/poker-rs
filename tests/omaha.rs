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

#[test]
fn omaha_hole_cards_rejects_duplicates() {
    let card = Card::new(Rank::Ace, Suit::Spades);
    let err = OmahaHoleCards::try_new(
        card,
        card,
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Clubs),
    )
    .unwrap_err();
    assert!(matches!(err, OmahaError::DuplicateHoleCards));
}

#[test]
fn omaha_hole_cards_as_array() {
    let cards = [
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Diamonds),
    ];
    let hole = hole(cards[0], cards[1], cards[2], cards[3]);
    assert_eq!(hole.as_array(), cards);
}

#[test]
fn omaha_hole_cards_from_str_valid() {
    let hole = "As Kh Qc Jd".parse::<OmahaHoleCards>().unwrap();
    let expected = [
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Diamonds),
    ];
    assert_eq!(hole.as_array(), expected);
}

#[test]
fn omaha_hole_cards_from_str_with_commas() {
    let hole = "As,Kh,Qc,Jd".parse::<OmahaHoleCards>().unwrap();
    let expected = [
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Diamonds),
    ];
    assert_eq!(hole.as_array(), expected);
}

#[test]
fn omaha_hole_cards_from_str_too_few() {
    let err = "As Kh Qc".parse::<OmahaHoleCards>().unwrap_err();
    assert!(matches!(err, OmahaError::HoleCount(3)));
}

#[test]
fn omaha_hole_cards_from_str_too_many() {
    let err = "As Kh Qc Jd Ts".parse::<OmahaHoleCards>().unwrap_err();
    assert!(matches!(err, OmahaError::HoleCount(5)));
}

#[test]
fn omaha_hole_cards_from_str_duplicates() {
    let err = "As As Kh Qc".parse::<OmahaHoleCards>().unwrap_err();
    assert!(matches!(err, OmahaError::DuplicateHoleCards));
}

#[test]
fn board_rejects_duplicate_cards() {
    use poker_rs::hand::HandError;
    let board_result = Board::try_new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Diamonds),
    ]);
    assert!(matches!(board_result, Err(HandError::DuplicateBoardCards)));
}

#[test]
fn omaha_straight_flush() {
    let board = Board::try_new(vec![
        Card::new(Rank::Nine, Suit::Hearts),
        Card::new(Rank::Eight, Suit::Hearts),
        Card::new(Rank::Seven, Suit::Hearts),
        Card::new(Rank::Two, Suit::Clubs),
        Card::new(Rank::Three, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::Jack, Suit::Hearts),
        Card::new(Rank::Ten, Suit::Hearts),
        Card::new(Rank::Six, Suit::Hearts),
        Card::new(Rank::Five, Suit::Hearts),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::StraightFlush));
}

#[test]
fn omaha_flush() {
    let board = Board::try_new(vec![
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Nine, Suit::Spades),
        Card::new(Rank::Two, Suit::Hearts),
        Card::new(Rank::Three, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Five, Suit::Hearts),
        Card::new(Rank::Four, Suit::Diamonds),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::Flush));
}

#[test]
fn omaha_straight() {
    let board = Board::try_new(vec![
        Card::new(Rank::Ten, Suit::Clubs),
        Card::new(Rank::Nine, Suit::Diamonds),
        Card::new(Rank::Eight, Suit::Hearts),
        Card::new(Rank::Two, Suit::Spades),
        Card::new(Rank::Three, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Five, Suit::Diamonds),
        Card::new(Rank::Four, Suit::Clubs),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::Straight));
}

#[test]
fn omaha_full_house() {
    let board = Board::try_new(vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Queen, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Two, Suit::Clubs),
    ])
    .unwrap();
    // Using Kh + Ace from hole, and KK QQ from board won't work since we need exactly 2 from hole
    // Best combination: Kh + Ace from hole, Kc Kd Q from board = Three Kings with Ace Queen
    // Let's try: Kh + Q? from hole and KK QQ from board to make full house
    let hole = hole(
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Three, Suit::Diamonds),
        Card::new(Rank::Four, Suit::Diamonds),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    // Using Kh + Qc from hole, Kc Kd Qs from board = KKK QQ = Full House
    assert!(matches!(eval.category, Category::FullHouse));
}

#[test]
fn omaha_two_pair() {
    let board = Board::try_new(vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Diamonds),
        Card::new(Rank::Jack, Suit::Hearts),
        Card::new(Rank::Two, Suit::Spades),
        Card::new(Rank::Three, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Five, Suit::Diamonds),
        Card::new(Rank::Four, Suit::Clubs),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::TwoPair));
}

#[test]
fn omaha_pair() {
    let board = Board::try_new(vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Diamonds),
        Card::new(Rank::Jack, Suit::Hearts),
        Card::new(Rank::Two, Suit::Spades),
        Card::new(Rank::Three, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Five, Suit::Spades),
        Card::new(Rank::Six, Suit::Diamonds),
        Card::new(Rank::Four, Suit::Clubs),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::Pair));
}

#[test]
fn omaha_high_card() {
    let board = Board::try_new(vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Diamonds),
        Card::new(Rank::Nine, Suit::Hearts),
        Card::new(Rank::Seven, Suit::Spades),
        Card::new(Rank::Five, Suit::Clubs),
    ])
    .unwrap();
    let hole = hole(
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Four, Suit::Diamonds),
        Card::new(Rank::Three, Suit::Clubs),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::HighCard));
}

#[test]
fn omaha_compare_different_hands() {
    let board = Board::try_new(vec![
        Card::new(Rank::King, Suit::Clubs),
        Card::new(Rank::Queen, Suit::Clubs),
        Card::new(Rank::Nine, Suit::Clubs),
        Card::new(Rank::Seven, Suit::Spades),
        Card::new(Rank::Two, Suit::Hearts),
    ])
    .unwrap();
    // Flush hand: needs 2 clubs from hole + 3 clubs from board
    let flush = hole(
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Clubs),
        Card::new(Rank::Three, Suit::Hearts),
        Card::new(Rank::Four, Suit::Spades),
    );
    // Pair hand: KK from board + any 2 from hole
    let pair = hole(
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::Five, Suit::Spades),
        Card::new(Rank::Six, Suit::Diamonds),
        Card::new(Rank::Eight, Suit::Diamonds),
    );
    let ord = compare_omaha(&flush, &pair, &board).unwrap();
    assert!(ord.is_gt());
}

#[test]
fn omaha_best_combination_selected() {
    // Board has three aces, we want to ensure we select the best 2+3 combination
    let board = Board::try_new(vec![
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Ace, Suit::Diamonds),
        Card::new(Rank::King, Suit::Hearts),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Queen, Suit::Clubs),
    ])
    .unwrap();
    // With As + Kd from hole and Ac Ad Kh from board = AAA KK = Full House
    let hole = hole(
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Two, Suit::Diamonds),
        Card::new(Rank::Three, Suit::Clubs),
    );
    let eval = evaluate_omaha(&hole, &board).unwrap();
    assert!(matches!(eval.category, Category::FullHouse));
}
