pub(crate) mod combinations;
pub(crate) mod detector;
pub(crate) mod hand_analysis;
pub(crate) mod rank_groups;
pub(crate) mod straight_info;
pub(crate) mod suit_info;

use crate::cards::{Card, Rank};
use crate::hand::{validate_holdem, Board, HandError, HoleCards};
use core::cmp::Ordering;

/// Compact, comparable hand strength. Higher is better.
/// Encodes category and ranked tiebreakers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct HandValue(u64);

/// Poker hand category from weakest to strongest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum Category {
    HighCard = 0,
    Pair = 1,
    TwoPair = 2,
    ThreeOfAKind = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    FourOfAKind = 7,
    StraightFlush = 8,
}

impl Category {
    pub const fn ordinal(self) -> u8 {
        self as u8
    }
}

/// Detailed evaluation result. `value` drives ordering.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Evaluation {
    pub category: Category,
    pub best_five: [Card; 5],
    value: HandValue,
}

impl Ord for Evaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Evaluation {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Evaluation {}

impl Evaluation {
    /// Return the packed comparable value for ordering/caching.
    pub const fn value(&self) -> HandValue {
        self.value
    }
}

impl HandValue {
    /// Return the packed comparable value.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Pack a category and five rank tiebreakers into a comparable value.
    /// Uses 6 bits per rank to be generous (supports up to 63).
    pub fn from_parts(category: Category, ranks_desc: &[Rank; 5]) -> Self {
        // Layout (most significant -> least):
        // [ category (8 bits) | r0 (6) | r1 (6) | r2 (6) | r3 (6) | r4 (6) | 10 zero bits ]
        // r0 is the primary tiebreaker and must be more significant than r1..r4.
        const CAT_SHIFT: u32 = 48; // put category in the high byte
        const RANK_STRIDE: u32 = 6;
        let mut v: u64 = (category as u64) << CAT_SHIFT;
        for (i, r) in ranks_desc.iter().enumerate() {
            // Place r0 just below the category, then r1, ...
            let offset = CAT_SHIFT - RANK_STRIDE * (i as u32 + 1);
            v |= (*r as u64) << offset;
        }
        HandValue(v)
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum EvalError {
    #[error("invalid hand: {0}")]
    InvalidHand(#[from] HandError),
    #[error("not enough cards to evaluate")]
    NotEnoughCards,
}

/// Evaluate a Hold'em hand given hole cards and a board.
/// Validates inputs, builds the 7-card set (2 hole + 5 board),
/// and returns the best five-card evaluation with category and tiebreaks.
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::evaluator::{evaluate_holdem, Category};
/// use poker_rs::hand::{Board, HoleCards};
///
/// let hole = HoleCards::try_new(
///     Card::new(Rank::Ace, Suit::Spades),
///     Card::new(Rank::Ace, Suit::Hearts),
/// ).unwrap();
/// let board = Board::try_new(vec![
///     Card::new(Rank::Queen, Suit::Clubs),
///     Card::new(Rank::Jack, Suit::Diamonds),
///     Card::new(Rank::Nine, Suit::Hearts),
///     Card::new(Rank::Three, Suit::Spades),
///     Card::new(Rank::Two, Suit::Clubs),
/// ]).unwrap();
/// let eval = evaluate_holdem(&hole, &board).unwrap();
/// assert_eq!(eval.category, Category::Pair);
/// ```
pub fn evaluate_holdem(hole: &HoleCards, board: &Board) -> Result<Evaluation, EvalError> {
    validate_holdem(hole, board)?;
    let board_cards = board.as_slice();
    if board_cards.len() < 5 {
        return Err(EvalError::NotEnoughCards);
    }
    // Build a 7-card set: two hole + 5 board
    let seven = [
        hole.first(),
        hole.second(),
        board_cards[0],
        board_cards[1],
        board_cards[2],
        board_cards[3],
        board_cards[4],
    ];
    Ok(evaluate_seven(&seven))
}

/// Evaluate exactly five cards; detects category and encodes tie-breakers.
pub fn evaluate_five(cards: &[Card; 5]) -> Evaluation {
    use detector::DETECTORS;
    use hand_analysis::HandAnalysis;

    // Build analysis once (sorted cards, rank counts, groups, flush/straight info)
    let analysis = HandAnalysis::new(cards);

    // Check categories in priority order (highest to lowest)
    for detector in DETECTORS.iter() {
        if detector.detect(&analysis) {
            return detector.build_evaluation(&analysis);
        }
    }

    // Unreachable: HighCard detector always matches as fallback
    unreachable!("HighCard detector should always match")
}

/// Evaluate seven cards (helper for Hold'em style 7-card evaluation).
/// Iterate all 21 five-card combinations from 7 and return the best by value.
pub fn evaluate_seven(cards: &[Card; 7]) -> Evaluation {
    use combinations::Combinations7Choose5;

    let mut best: Option<Evaluation> = None;

    for indices in Combinations7Choose5::new() {
        let hand = [
            cards[indices[0]],
            cards[indices[1]],
            cards[indices[2]],
            cards[indices[3]],
            cards[indices[4]],
        ];
        let eval = evaluate_five(&hand);

        if best.as_ref().map_or(true, |b| eval > *b) {
            best = Some(eval);
        }
    }

    best.unwrap_or_else(|| evaluate_five(&[cards[0], cards[1], cards[2], cards[3], cards[4]]))
}

/// Compare two Hold'em hands on a shared board. Returns the ordering or a validation error.
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::evaluator::compare_holdem;
/// use poker_rs::hand::{Board, HoleCards};
/// use std::cmp::Ordering;
///
/// let board = Board::try_new(vec![
///     Card::new(Rank::Queen, Suit::Clubs),
///     Card::new(Rank::Jack, Suit::Diamonds),
///     Card::new(Rank::Nine, Suit::Hearts),
///     Card::new(Rank::Three, Suit::Spades),
///     Card::new(Rank::Two, Suit::Clubs),
/// ]).unwrap();
/// let a = HoleCards::try_new(
///     Card::new(Rank::Ace, Suit::Spades),
///     Card::new(Rank::Ace, Suit::Hearts),
/// ).unwrap();
/// let b = HoleCards::try_new(
///     Card::new(Rank::King, Suit::Spades),
///     Card::new(Rank::King, Suit::Hearts),
/// ).unwrap();
/// let ord = compare_holdem(&a, &b, &board).unwrap();
/// assert_eq!(ord, Ordering::Greater);
/// ```
pub fn compare_holdem(a: &HoleCards, b: &HoleCards, board: &Board) -> Result<Ordering, EvalError> {
    let va = evaluate_holdem(a, board)?;
    let vb = evaluate_holdem(b, board)?;
    Ok(va.cmp(&vb))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Rank, Suit};

    fn hole(a: Card, b: Card) -> HoleCards {
        HoleCards::try_new(a, b).expect("valid hole cards")
    }

    #[test]
    fn not_enough_cards_errors() {
        let hole = hole(Card::new(Rank::Ace, Suit::Spades), Card::new(Rank::King, Suit::Spades));
        let board = Board::new(vec![Card::new(Rank::Two, Suit::Clubs)]);
        let err = evaluate_holdem(&hole, &board).unwrap_err();
        assert!(matches!(err, EvalError::NotEnoughCards));
    }

    #[test]
    fn compare_errors_with_short_board() {
        let a = hole(Card::new(Rank::Ace, Suit::Spades), Card::new(Rank::King, Suit::Spades));
        let b = hole(Card::new(Rank::Two, Suit::Clubs), Card::new(Rank::Three, Suit::Clubs));
        let board = Board::new(vec![Card::new(Rank::Two, Suit::Hearts)]);
        let err = compare_holdem(&a, &b, &board).unwrap_err();
        assert!(matches!(err, EvalError::NotEnoughCards));
    }

    #[test]
    fn evaluate_five_categories() {
        // Straight flush
        let sf = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Ten, Suit::Spades),
        ];
        let e = evaluate_five(&sf);
        assert!(matches!(e.category, Category::StraightFlush));

        // Four of a kind
        let quads = [
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Two, Suit::Spades),
        ];
        let e = evaluate_five(&quads);
        assert!(matches!(e.category, Category::FourOfAKind));

        // Full house
        let fh = [
            Card::new(Rank::Ten, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Diamonds),
            Card::new(Rank::Ten, Suit::Hearts),
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
        ];
        let e = evaluate_five(&fh);
        assert!(matches!(e.category, Category::FullHouse));

        // Flush
        let fl = [
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Nine, Suit::Hearts),
            Card::new(Rank::Seven, Suit::Hearts),
            Card::new(Rank::Three, Suit::Hearts),
            Card::new(Rank::Two, Suit::Hearts),
        ];
        let e = evaluate_five(&fl);
        assert!(matches!(e.category, Category::Flush));

        // Straight (wheel)
        let st = [
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Three, Suit::Hearts),
            Card::new(Rank::Four, Suit::Spades),
            Card::new(Rank::Five, Suit::Clubs),
        ];
        let e = evaluate_five(&st);
        assert!(matches!(e.category, Category::Straight));

        // Trips
        let tk = [
            Card::new(Rank::Queen, Suit::Clubs),
            Card::new(Rank::Queen, Suit::Diamonds),
            Card::new(Rank::Queen, Suit::Hearts),
            Card::new(Rank::Nine, Suit::Spades),
            Card::new(Rank::Two, Suit::Clubs),
        ];
        let e = evaluate_five(&tk);
        assert!(matches!(e.category, Category::ThreeOfAKind));

        // Two pair
        let tp = [
            Card::new(Rank::Jack, Suit::Clubs),
            Card::new(Rank::Jack, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Clubs),
            Card::new(Rank::Nine, Suit::Hearts),
            Card::new(Rank::Two, Suit::Spades),
        ];
        let e = evaluate_five(&tp);
        assert!(matches!(e.category, Category::TwoPair));

        // Pair
        let pr = [
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Nine, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
        ];
        let e = evaluate_five(&pr);
        assert!(matches!(e.category, Category::Pair));

        // High card
        let hi = [
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
        ];
        let e = evaluate_five(&hi);
        assert!(matches!(e.category, Category::HighCard));
    }
}
