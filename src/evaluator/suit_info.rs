use crate::cards::{Card, Suit};

/// Information about whether all cards share the same suit (flush).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuitInfo {
    pub is_flush: bool,
    pub flush_suit: Option<Suit>,
}

impl SuitInfo {
    /// Detect if all 5 cards have the same suit.
    pub fn detect(cards: &[Card; 5]) -> Self {
        let first_suit = cards[0].suit();
        let all_same = cards.iter().all(|c| c.suit() == first_suit);

        if all_same {
            SuitInfo { is_flush: true, flush_suit: Some(first_suit) }
        } else {
            SuitInfo { is_flush: false, flush_suit: None }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Rank;

    #[test]
    fn test_flush() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Nine, Suit::Spades),
        ];
        let info = SuitInfo::detect(&cards);
        assert!(info.is_flush);
        assert_eq!(info.flush_suit, Some(Suit::Spades));
    }

    #[test]
    fn test_not_flush() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Nine, Suit::Spades),
        ];
        let info = SuitInfo::detect(&cards);
        assert!(!info.is_flush);
        assert_eq!(info.flush_suit, None);
    }

    #[test]
    fn test_all_clubs() {
        let cards = [
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Clubs),
            Card::new(Rank::Four, Suit::Clubs),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Clubs),
        ];
        let info = SuitInfo::detect(&cards);
        assert!(info.is_flush);
        assert_eq!(info.flush_suit, Some(Suit::Clubs));
    }
}
