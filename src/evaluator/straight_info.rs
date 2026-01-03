use crate::cards::Rank;

/// Information about whether a hand contains a straight and its top rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StraightInfo {
    pub is_straight: bool,
    pub top_rank: Option<Rank>,
}

impl StraightInfo {
    /// Detect a straight from an array of 5 ranks.
    /// Handles both regular straights and the wheel (A-2-3-4-5).
    pub fn detect(ranks: &[Rank; 5]) -> Self {
        // Sort ranks descending
        let mut sorted_ranks = *ranks;
        sorted_ranks.sort_by(|a, b| b.cmp(a));

        // Check for regular straight (5 consecutive ranks descending)
        let is_consecutive =
            (0..4).all(|i| sorted_ranks[i].value() == sorted_ranks[i + 1].value() + 1);

        if is_consecutive {
            return StraightInfo { is_straight: true, top_rank: Some(sorted_ranks[0]) };
        }

        // Check for wheel (A-2-3-4-5): Ace high, then 5-4-3-2
        if sorted_ranks[0] == Rank::Ace
            && sorted_ranks[1] == Rank::Five
            && sorted_ranks[2] == Rank::Four
            && sorted_ranks[3] == Rank::Three
            && sorted_ranks[4] == Rank::Two
        {
            return StraightInfo {
                is_straight: true,
                top_rank: Some(Rank::Five), // In wheel, Five is the top rank
            };
        }

        StraightInfo { is_straight: false, top_rank: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_straight() {
        let ranks = [Rank::King, Rank::Queen, Rank::Jack, Rank::Ten, Rank::Nine];
        let info = StraightInfo::detect(&ranks);
        assert!(info.is_straight);
        assert_eq!(info.top_rank, Some(Rank::King));
    }

    #[test]
    fn test_ace_high_straight() {
        let ranks = [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten];
        let info = StraightInfo::detect(&ranks);
        assert!(info.is_straight);
        assert_eq!(info.top_rank, Some(Rank::Ace));
    }

    #[test]
    fn test_wheel() {
        let ranks = [Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five];
        let info = StraightInfo::detect(&ranks);
        assert!(info.is_straight);
        assert_eq!(info.top_rank, Some(Rank::Five)); // Five is high in wheel
    }

    #[test]
    fn test_low_straight() {
        let ranks = [Rank::Six, Rank::Five, Rank::Four, Rank::Three, Rank::Two];
        let info = StraightInfo::detect(&ranks);
        assert!(info.is_straight);
        assert_eq!(info.top_rank, Some(Rank::Six));
    }

    #[test]
    fn test_not_straight() {
        let ranks = [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine];
        let info = StraightInfo::detect(&ranks);
        assert!(!info.is_straight);
        assert_eq!(info.top_rank, None);
    }

    #[test]
    fn test_not_straight_pair() {
        let ranks = [Rank::Ace, Rank::Ace, Rank::King, Rank::Queen, Rank::Jack];
        let info = StraightInfo::detect(&ranks);
        assert!(!info.is_straight);
        assert_eq!(info.top_rank, None);
    }

    #[test]
    fn test_unsorted_input() {
        // Input can be in any order
        let ranks = [Rank::Nine, Rank::King, Rank::Ten, Rank::Jack, Rank::Queen];
        let info = StraightInfo::detect(&ranks);
        assert!(info.is_straight);
        assert_eq!(info.top_rank, Some(Rank::King));
    }
}
