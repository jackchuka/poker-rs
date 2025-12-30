use crate::cards::{parse_cards, Card};
use crate::evaluator::{evaluate_five, Evaluation};
use crate::hand::{Board, HandError};
use core::cmp::Ordering;
use std::collections::HashSet;
use std::str::FromStr;

/// Omaha hole cards: exactly four private cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmahaHoleCards(Card, Card, Card, Card);

impl OmahaHoleCards {
    /// ```
    /// use poker_rs::cards::{Card, Rank, Suit};
    /// use poker_rs::variants::omaha::OmahaHoleCards;
    ///
    /// let hole = OmahaHoleCards::try_new(
    ///     Card::new(Rank::Ace, Suit::Spades),
    ///     Card::new(Rank::King, Suit::Spades),
    ///     Card::new(Rank::Queen, Suit::Hearts),
    ///     Card::new(Rank::Jack, Suit::Hearts),
    /// ).unwrap();
    /// assert_eq!(hole.as_array().len(), 4);
    /// ```
    pub fn try_new(a: Card, b: Card, c: Card, d: Card) -> Result<Self, OmahaError> {
        let mut set = HashSet::new();
        for card in [a, b, c, d] {
            if !set.insert(card) {
                return Err(OmahaError::DuplicateHoleCards);
            }
        }
        Ok(Self(a, b, c, d))
    }

    pub fn as_array(&self) -> [Card; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl FromStr for OmahaHoleCards {
    type Err = OmahaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = parse_cards(s).map_err(|e| OmahaError::CardParse(e.to_string()))?;
        if cards.len() != 4 {
            return Err(OmahaError::HoleCount(cards.len()));
        }
        OmahaHoleCards::try_new(cards[0], cards[1], cards[2], cards[3])
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum OmahaError {
    #[error("expected exactly four hole cards, got {0}")]
    HoleCount(usize),
    #[error("duplicate cards in hole cards")]
    DuplicateHoleCards,
    #[error("board must have exactly five cards, got {0}")]
    BoardCount(usize),
    #[error("duplicate cards on board")]
    DuplicateBoardCards,
    #[error("hole cards overlap with board")]
    Overlap,
    #[error("board error: {0}")]
    Board(#[from] HandError),
    #[error("card parse error: {0}")]
    CardParse(String),
}

/// Validate that a 4-card hole and board form a valid Omaha state.
pub fn validate_omaha(hole: &OmahaHoleCards, board: &Board) -> Result<(), OmahaError> {
    if board.len() != 5 {
        return Err(OmahaError::BoardCount(board.len()));
    }
    let board_cards = board.as_slice();
    let mut set: HashSet<Card> = HashSet::with_capacity(board_cards.len());
    for &card in board_cards {
        if !set.insert(card) {
            return Err(OmahaError::DuplicateBoardCards);
        }
    }
    for card in hole.as_array() {
        if set.contains(&card) {
            return Err(OmahaError::Overlap);
        }
    }
    Ok(())
}

/// Evaluate an Omaha hand with the rule "use exactly 2 hole + 3 board cards".
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::hand::Board;
/// use poker_rs::variants::omaha::{evaluate_omaha, OmahaHoleCards};
///
/// let hole = OmahaHoleCards::try_new(
///     Card::new(Rank::Ace, Suit::Spades),
///     Card::new(Rank::King, Suit::Spades),
///     Card::new(Rank::Queen, Suit::Hearts),
///     Card::new(Rank::Jack, Suit::Hearts),
/// ).unwrap();
/// let board = Board::try_new(vec![
///     Card::new(Rank::Ten, Suit::Clubs),
///     Card::new(Rank::Nine, Suit::Diamonds),
///     Card::new(Rank::Three, Suit::Hearts),
///     Card::new(Rank::Two, Suit::Spades),
///     Card::new(Rank::Four, Suit::Clubs),
/// ]).unwrap();
///
/// let eval = evaluate_omaha(&hole, &board).unwrap();
/// println!("Category: {:?}", eval.category);
/// ```
pub fn evaluate_omaha(hole: &OmahaHoleCards, board: &Board) -> Result<Evaluation, OmahaError> {
    validate_omaha(hole, board)?;
    let hole_cards = hole.as_array();
    let board_cards = board.as_slice();

    let mut best: Option<Evaluation> = None;
    for i in 0..3 {
        for j in (i + 1)..4 {
            for a in 0..3 {
                for b in (a + 1)..4 {
                    for c in (b + 1)..5 {
                        let hand = [
                            hole_cards[i],
                            hole_cards[j],
                            board_cards[a],
                            board_cards[b],
                            board_cards[c],
                        ];
                        let eval = evaluate_five(&hand);
                        if best.map_or(true, |b| eval > b) {
                            best = Some(eval);
                        }
                    }
                }
            }
        }
    }
    Ok(best.unwrap_or_else(|| {
        let hand = [hole_cards[0], hole_cards[1], board_cards[0], board_cards[1], board_cards[2]];
        evaluate_five(&hand)
    }))
}

/// Compare two Omaha hands on a shared board.
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::hand::Board;
/// use poker_rs::variants::omaha::{compare_omaha, OmahaHoleCards};
///
/// let board = Board::try_new(vec![
///     Card::new(Rank::Queen, Suit::Clubs),
///     Card::new(Rank::Jack, Suit::Diamonds),
///     Card::new(Rank::Nine, Suit::Hearts),
///     Card::new(Rank::Three, Suit::Spades),
///     Card::new(Rank::Two, Suit::Clubs),
/// ]).unwrap();
/// let a = OmahaHoleCards::try_new(
///     Card::new(Rank::Ace, Suit::Spades),
///     Card::new(Rank::Ace, Suit::Hearts),
///     Card::new(Rank::Five, Suit::Diamonds),
///     Card::new(Rank::Four, Suit::Clubs),
/// ).unwrap();
/// let b = OmahaHoleCards::try_new(
///     Card::new(Rank::King, Suit::Spades),
///     Card::new(Rank::King, Suit::Hearts),
///     Card::new(Rank::Eight, Suit::Diamonds),
///     Card::new(Rank::Seven, Suit::Clubs),
/// ).unwrap();
/// let ord = compare_omaha(&a, &b, &board).unwrap();
/// assert!(ord.is_gt());
/// ```
pub fn compare_omaha(
    a: &OmahaHoleCards,
    b: &OmahaHoleCards,
    board: &Board,
) -> Result<Ordering, OmahaError> {
    let va = evaluate_omaha(a, board)?;
    let vb = evaluate_omaha(b, board)?;
    Ok(va.cmp(&vb))
}
