use super::rank_groups::RankGroups;
use super::straight_info::StraightInfo;
use super::suit_info::SuitInfo;
use crate::cards::{Card, Rank};
use crate::evaluator::{Category, Evaluation, HandValue};

/// Pre-computed analysis of a 5-card hand.
/// Built once and shared by all category detectors.
#[derive(Debug, Clone)]
pub struct HandAnalysis {
    pub sorted_cards: [Card; 5],
    pub ranks: [Rank; 5],
    /// Raw rank counts array (kept for potential future use and debugging)
    #[allow(dead_code)]
    pub rank_counts: [u8; 15],
    pub rank_groups: RankGroups,
    pub suit_info: SuitInfo,
    pub straight_info: StraightInfo,
}

impl HandAnalysis {
    /// Analyze a 5-card hand, computing all properties needed for evaluation.
    pub fn new(cards: &[Card; 5]) -> Self {
        // Sort cards by rank descending, then by suit descending
        let mut sorted_cards = *cards;
        sorted_cards.sort_by(|a, b| b.rank().cmp(&a.rank()).then(b.suit().cmp(&a.suit())));

        // Extract ranks
        let ranks = [
            sorted_cards[0].rank(),
            sorted_cards[1].rank(),
            sorted_cards[2].rank(),
            sorted_cards[3].rank(),
            sorted_cards[4].rank(),
        ];

        // Count rank frequencies
        let mut rank_counts = [0u8; 15];
        for &rank in ranks.iter() {
            rank_counts[rank.value() as usize] += 1;
        }

        // Build domain objects
        let rank_groups = RankGroups::from_counts(&rank_counts);
        let suit_info = SuitInfo::detect(&sorted_cards);
        let straight_info = StraightInfo::detect(&ranks);

        Self { sorted_cards, ranks, rank_counts, rank_groups, suit_info, straight_info }
    }

    /// Build an Evaluation from a category and tiebreak ranks.
    pub fn build_evaluation(&self, category: Category, tiebreak: [Rank; 5]) -> Evaluation {
        let value = HandValue::from_parts(category, &tiebreak);
        Evaluation { category, best_five: self.sorted_cards, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Suit;

    #[test]
    fn test_royal_flush_analysis() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Ten, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert!(analysis.suit_info.is_flush);
        assert!(analysis.straight_info.is_straight);
        assert_eq!(analysis.straight_info.top_rank, Some(Rank::Ace));
        assert_eq!(analysis.rank_groups.quad(), None);
        assert_eq!(analysis.rank_groups.trips(), None);
        assert_eq!(analysis.rank_groups.pairs(), vec![]);
    }

    #[test]
    fn test_quads_analysis() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::King, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert_eq!(analysis.rank_groups.quad(), Some(Rank::Ace));
        assert_eq!(analysis.rank_groups.kickers(), vec![Rank::King]);
        assert!(!analysis.suit_info.is_flush);
        assert!(!analysis.straight_info.is_straight);
    }

    #[test]
    fn test_full_house_analysis() {
        let cards = [
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Queen, Suit::Clubs),
            Card::new(Rank::Queen, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert!(analysis.rank_groups.has_full_house());
        assert_eq!(analysis.rank_groups.trips(), Some(Rank::King));
        assert_eq!(analysis.rank_groups.pairs(), vec![Rank::Queen]);
    }

    #[test]
    fn test_flush_analysis() {
        let cards = [
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Jack, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Diamonds),
            Card::new(Rank::Five, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Diamonds),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert!(analysis.suit_info.is_flush);
        assert_eq!(analysis.suit_info.flush_suit, Some(Suit::Diamonds));
        assert!(!analysis.straight_info.is_straight);
    }

    #[test]
    fn test_straight_analysis() {
        let cards = [
            Card::new(Rank::Nine, Suit::Spades),
            Card::new(Rank::Eight, Suit::Hearts),
            Card::new(Rank::Seven, Suit::Diamonds),
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::Five, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert!(analysis.straight_info.is_straight);
        assert_eq!(analysis.straight_info.top_rank, Some(Rank::Nine));
        assert!(!analysis.suit_info.is_flush);
    }

    #[test]
    fn test_two_pair_analysis() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Queen, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        let pairs = analysis.rank_groups.pairs();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], Rank::Ace);
        assert_eq!(pairs[1], Rank::King);
        assert_eq!(analysis.rank_groups.kickers(), vec![Rank::Queen]);
    }

    #[test]
    fn test_one_pair_analysis() {
        let cards = [
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Nine, Suit::Diamonds),
            Card::new(Rank::Seven, Suit::Clubs),
            Card::new(Rank::Three, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert_eq!(analysis.rank_groups.pairs(), vec![Rank::Jack]);
        let kickers = analysis.rank_groups.kickers();
        assert_eq!(kickers.len(), 3);
        assert_eq!(kickers[0], Rank::Nine);
        assert_eq!(kickers[1], Rank::Seven);
        assert_eq!(kickers[2], Rank::Three);
    }

    #[test]
    fn test_high_card_analysis() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert_eq!(analysis.rank_groups.quad(), None);
        assert_eq!(analysis.rank_groups.trips(), None);
        assert_eq!(analysis.rank_groups.pairs(), vec![]);
        assert_eq!(analysis.rank_groups.kickers().len(), 5);
    }

    #[test]
    fn test_wheel_straight_analysis() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Three, Suit::Diamonds),
            Card::new(Rank::Four, Suit::Clubs),
            Card::new(Rank::Five, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        assert!(analysis.straight_info.is_straight);
        assert_eq!(analysis.straight_info.top_rank, Some(Rank::Five)); // Five is high in wheel
    }

    #[test]
    fn test_cards_sorted_descending() {
        let cards = [
            Card::new(Rank::Three, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Five, Suit::Diamonds),
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Nine, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);

        // Should be sorted: A, K, 9, 5, 3
        assert_eq!(analysis.sorted_cards[0].rank(), Rank::Ace);
        assert_eq!(analysis.sorted_cards[1].rank(), Rank::King);
        assert_eq!(analysis.sorted_cards[2].rank(), Rank::Nine);
        assert_eq!(analysis.sorted_cards[3].rank(), Rank::Five);
        assert_eq!(analysis.sorted_cards[4].rank(), Rank::Three);
    }
}
