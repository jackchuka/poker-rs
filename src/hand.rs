use crate::cards::{parse_cards, Card};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HandError {
    #[error("duplicate cards in hole cards")]
    DuplicateHoleCards,
    #[error("too many board cards: {0}")]
    TooManyBoardCards(usize),
    #[error("duplicate cards on board")]
    DuplicateBoardCards,
    #[error("hole cards overlap with board")]
    Overlap,
    #[error("expected exactly two hole cards, got {0}")]
    HoleCount(usize),
    #[error("card parse error: {0}")]
    CardParse(String),
}

/// A player's two private hole cards.
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::hand::HoleCards;
///
/// let hole = HoleCards::try_new(
///     Card::new(Rank::Ace, Suit::Spades),
///     Card::new(Rank::King, Suit::Spades),
/// ).unwrap();
/// assert_eq!(hole.as_array().len(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoleCards(Card, Card);

impl HoleCards {
    /// Return the first (left) hole card.
    pub fn first(&self) -> Card {
        self.0
    }

    /// Return the second (right) hole card.
    pub fn second(&self) -> Card {
        self.1
    }

    /// Return both hole cards as a fixed array.
    pub fn as_array(&self) -> [Card; 2] {
        [self.0, self.1]
    }

    pub fn try_new(a: Card, b: Card) -> Result<Self, HandError> {
        if a == b {
            return Err(HandError::DuplicateHoleCards);
        }
        Ok(Self(a, b))
    }

    pub fn from_slice(slice: &[Card]) -> Result<Self, HandError> {
        if slice.len() != 2 {
            return Err(HandError::HoleCount(slice.len()));
        }
        Self::try_new(slice[0], slice[1])
    }
}

impl FromStr for HoleCards {
    type Err = HandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = parse_cards(s).map_err(|e| HandError::CardParse(e.to_string()))?;
        Self::from_slice(&cards)
    }
}

/// Community cards on the board (flop, turn, river).
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::hand::Board;
///
/// let board = Board::try_new(vec![
///     Card::new(Rank::Two, Suit::Clubs),
///     Card::new(Rank::Three, Suit::Clubs),
///     Card::new(Rank::Four, Suit::Clubs),
/// ]).unwrap();
/// assert_eq!(board.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    cards: Vec<Card>,
}

impl Board {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    pub fn try_new(cards: Vec<Card>) -> Result<Self, HandError> {
        if cards.len() > 5 {
            return Err(HandError::TooManyBoardCards(cards.len()));
        }
        let set: HashSet<Card> = cards.iter().copied().collect();
        if set.len() != cards.len() {
            return Err(HandError::DuplicateBoardCards);
        }
        Ok(Self { cards })
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn as_slice(&self) -> &[Card] {
        &self.cards
    }

    pub(crate) fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub(crate) fn extend<I>(&mut self, cards: I)
    where
        I: IntoIterator<Item = Card>,
    {
        self.cards.extend(cards);
    }
}

impl FromStr for Board {
    type Err = HandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = parse_cards(s).map_err(|e| HandError::CardParse(e.to_string()))?;
        Board::try_new(cards)
    }
}

/// Validate that a pair of hole cards and board form a valid Hold'em state.
/// Allows 0..=5 board cards (useful during gameplay). Ensures uniqueness across all cards.
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
/// use poker_rs::hand::{Board, HoleCards, validate_holdem};
///
/// let hole = HoleCards::try_new(
///     Card::new(Rank::Ace, Suit::Spades),
///     Card::new(Rank::King, Suit::Spades),
/// ).unwrap();
/// let board = Board::try_new(vec![
///     Card::new(Rank::Two, Suit::Clubs),
///     Card::new(Rank::Three, Suit::Clubs),
///     Card::new(Rank::Four, Suit::Clubs),
/// ]).unwrap();
/// validate_holdem(&hole, &board).unwrap();
/// ```
pub fn validate_holdem(hole: &HoleCards, board: &Board) -> Result<(), HandError> {
    if board.len() > 5 {
        return Err(HandError::TooManyBoardCards(board.len()));
    }
    // Ensure board has no duplicates (in case created via `new`)
    let set: HashSet<Card> = board.as_slice().iter().copied().collect();
    if set.len() != board.len() {
        return Err(HandError::DuplicateBoardCards);
    }
    // Ensure no overlap between hole and board
    if set.contains(&hole.first()) || set.contains(&hole.second()) {
        return Err(HandError::Overlap);
    }
    // Ensure hole cards are distinct
    if hole.first() == hole.second() {
        return Err(HandError::DuplicateHoleCards);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Rank, Suit};

    #[test]
    fn board_len_and_empty_work() {
        let b = Board::new(vec![Card::new(Rank::Ace, Suit::Spades)]);
        assert_eq!(b.len(), 1);
        assert!(!b.is_empty());
    }

    #[test]
    fn hole_cards_must_be_distinct() {
        let a = Card::new(Rank::Ace, Suit::Spades);
        assert!(matches!(HoleCards::try_new(a, a), Err(HandError::DuplicateHoleCards)));
    }

    #[test]
    fn board_try_new_checks_limits_and_dupes() {
        // Too many
        let cards = vec![
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Clubs),
            Card::new(Rank::Four, Suit::Clubs),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Clubs),
        ];
        assert!(matches!(Board::try_new(cards), Err(HandError::TooManyBoardCards(6))));

        // Duplicates
        let cards = vec![Card::new(Rank::Two, Suit::Clubs), Card::new(Rank::Two, Suit::Clubs)];
        assert!(matches!(Board::try_new(cards), Err(HandError::DuplicateBoardCards)));
    }

    #[test]
    fn validate_holdem_catches_overlap() {
        let a = Card::new(Rank::Ace, Suit::Spades);
        let k = Card::new(Rank::King, Suit::Spades);
        let hole = HoleCards::try_new(a, k).unwrap();
        let board = Board::new(vec![
            a,
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Clubs),
        ]);
        assert!(matches!(validate_holdem(&hole, &board), Err(HandError::Overlap)));
    }

    #[test]
    fn parsing_interfaces_work() {
        let hole: HoleCards = "As Kd".parse().unwrap();
        assert_eq!(hole.first(), Card::new(Rank::Ace, Suit::Spades));
        assert_eq!(hole.second(), Card::new(Rank::King, Suit::Diamonds));

        let board: Board = "2c, 3c 4c".parse().unwrap();
        assert_eq!(board.len(), 3);
    }
}
