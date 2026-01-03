use super::hand_analysis::HandAnalysis;
use crate::cards::Rank;
use crate::evaluator::{Category, Evaluation};

/// Strategy pattern: each category detector knows how to detect and build its evaluation.
pub trait CategoryDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool;
    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation;
}

// ============================================================================
// Detector Implementations (in priority order: highest to lowest)
// ============================================================================

/// Straight Flush: Five consecutive ranks, all same suit
pub struct StraightFlushDetector;

impl CategoryDetector for StraightFlushDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.suit_info.is_flush && analysis.straight_info.is_straight
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let top_rank = analysis.straight_info.top_rank.unwrap();
        let tiebreak = [top_rank, Rank::Two, Rank::Two, Rank::Two, Rank::Two];
        analysis.build_evaluation(Category::StraightFlush, tiebreak)
    }
}

/// Four of a Kind: Four cards of the same rank
pub struct FourOfAKindDetector;

impl CategoryDetector for FourOfAKindDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.rank_groups.quad().is_some()
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let quad_rank = analysis.rank_groups.quad().unwrap();
        let kicker = analysis.rank_groups.kickers()[0];
        let tiebreak = [quad_rank, kicker, Rank::Two, Rank::Two, Rank::Two];
        analysis.build_evaluation(Category::FourOfAKind, tiebreak)
    }
}

/// Full House: Three of a kind plus a pair
pub struct FullHouseDetector;

impl CategoryDetector for FullHouseDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.rank_groups.has_full_house()
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let trips = analysis.rank_groups.trips().unwrap();
        let pair = analysis.rank_groups.pairs()[0];
        let tiebreak = [trips, pair, Rank::Two, Rank::Two, Rank::Two];
        analysis.build_evaluation(Category::FullHouse, tiebreak)
    }
}

/// Flush: All five cards of the same suit
pub struct FlushDetector;

impl CategoryDetector for FlushDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.suit_info.is_flush
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        // All 5 cards are kickers in flush ranking
        let tiebreak = analysis.ranks;
        analysis.build_evaluation(Category::Flush, tiebreak)
    }
}

/// Straight: Five consecutive ranks (not all same suit)
pub struct StraightDetector;

impl CategoryDetector for StraightDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.straight_info.is_straight
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let top_rank = analysis.straight_info.top_rank.unwrap();
        let tiebreak = [top_rank, Rank::Two, Rank::Two, Rank::Two, Rank::Two];
        analysis.build_evaluation(Category::Straight, tiebreak)
    }
}

/// Three of a Kind: Three cards of the same rank
pub struct ThreeOfAKindDetector;

impl CategoryDetector for ThreeOfAKindDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.rank_groups.trips().is_some() && !analysis.rank_groups.has_full_house()
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let trips = analysis.rank_groups.trips().unwrap();
        let kickers = analysis.rank_groups.kickers();
        let tiebreak = [trips, kickers[0], kickers[1], Rank::Two, Rank::Two];
        analysis.build_evaluation(Category::ThreeOfAKind, tiebreak)
    }
}

/// Two Pair: Two pairs of cards
pub struct TwoPairDetector;

impl CategoryDetector for TwoPairDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.rank_groups.pairs().len() == 2
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let pairs = analysis.rank_groups.pairs();
        let kicker = analysis.rank_groups.kickers()[0];
        let tiebreak = [pairs[0], pairs[1], kicker, Rank::Two, Rank::Two];
        analysis.build_evaluation(Category::TwoPair, tiebreak)
    }
}

/// One Pair: Two cards of the same rank
pub struct OnePairDetector;

impl CategoryDetector for OnePairDetector {
    fn detect(&self, analysis: &HandAnalysis) -> bool {
        analysis.rank_groups.pairs().len() == 1
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let pair = analysis.rank_groups.pairs()[0];
        let kickers = analysis.rank_groups.kickers();
        let tiebreak = [pair, kickers[0], kickers[1], kickers[2], Rank::Two];
        analysis.build_evaluation(Category::Pair, tiebreak)
    }
}

/// High Card: No matching ranks or sequences
pub struct HighCardDetector;

impl CategoryDetector for HighCardDetector {
    fn detect(&self, _analysis: &HandAnalysis) -> bool {
        true // Always matches as fallback
    }

    fn build_evaluation(&self, analysis: &HandAnalysis) -> Evaluation {
        let tiebreak = analysis.ranks;
        analysis.build_evaluation(Category::HighCard, tiebreak)
    }
}

// ============================================================================
// Static detector list (in priority order)
// ============================================================================

pub const DETECTORS: [&dyn CategoryDetector; 9] = [
    &StraightFlushDetector,
    &FourOfAKindDetector,
    &FullHouseDetector,
    &FlushDetector,
    &StraightDetector,
    &ThreeOfAKindDetector,
    &TwoPairDetector,
    &OnePairDetector,
    &HighCardDetector,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Suit};

    #[test]
    fn test_straight_flush_detector() {
        let cards = [
            Card::new(Rank::Nine, Suit::Hearts),
            Card::new(Rank::Eight, Suit::Hearts),
            Card::new(Rank::Seven, Suit::Hearts),
            Card::new(Rank::Six, Suit::Hearts),
            Card::new(Rank::Five, Suit::Hearts),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = StraightFlushDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::StraightFlush);
    }

    #[test]
    fn test_four_of_a_kind_detector() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::King, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = FourOfAKindDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::FourOfAKind);
    }

    #[test]
    fn test_full_house_detector() {
        let cards = [
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Queen, Suit::Clubs),
            Card::new(Rank::Queen, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = FullHouseDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::FullHouse);
    }

    #[test]
    fn test_flush_detector() {
        let cards = [
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Jack, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Diamonds),
            Card::new(Rank::Five, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Diamonds),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = FlushDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::Flush);
    }

    #[test]
    fn test_straight_detector() {
        let cards = [
            Card::new(Rank::Nine, Suit::Spades),
            Card::new(Rank::Eight, Suit::Hearts),
            Card::new(Rank::Seven, Suit::Diamonds),
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::Five, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = StraightDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::Straight);
    }

    #[test]
    fn test_three_of_a_kind_detector() {
        let cards = [
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = ThreeOfAKindDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::ThreeOfAKind);
    }

    #[test]
    fn test_two_pair_detector() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Queen, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = TwoPairDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::TwoPair);
    }

    #[test]
    fn test_one_pair_detector() {
        let cards = [
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Nine, Suit::Diamonds),
            Card::new(Rank::Seven, Suit::Clubs),
            Card::new(Rank::Three, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = OnePairDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::Pair);
    }

    #[test]
    fn test_high_card_detector() {
        let cards = [
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Spades),
        ];
        let analysis = HandAnalysis::new(&cards);
        let detector = HighCardDetector;

        assert!(detector.detect(&analysis));
        let eval = detector.build_evaluation(&analysis);
        assert_eq!(eval.category, Category::HighCard);
    }

    #[test]
    fn test_detector_priority_straight_flush_over_flush() {
        let cards = [
            Card::new(Rank::Nine, Suit::Hearts),
            Card::new(Rank::Eight, Suit::Hearts),
            Card::new(Rank::Seven, Suit::Hearts),
            Card::new(Rank::Six, Suit::Hearts),
            Card::new(Rank::Five, Suit::Hearts),
        ];
        let analysis = HandAnalysis::new(&cards);

        // Both straight and flush detectors would match, but straight flush should win
        assert!(StraightFlushDetector.detect(&analysis));
        assert!(FlushDetector.detect(&analysis));
        assert!(StraightDetector.detect(&analysis));
    }
}
