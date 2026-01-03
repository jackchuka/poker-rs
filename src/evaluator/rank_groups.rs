use crate::cards::Rank;

/// Groups ranks by their frequency in a hand, sorted by (count desc, rank desc).
///
/// Example: AAAKQ groups as [(Ace, 3), (King, 1), (Queen, 1)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankGroups {
    groups: Vec<(Rank, u8)>,
}

impl RankGroups {
    /// Create RankGroups from a rank count array.
    /// The array should be indexed by rank value (2-14).
    pub fn from_counts(rank_counts: &[u8; 15]) -> Self {
        let mut groups = Vec::new();

        for rank in Rank::ALL.iter().copied() {
            let count = rank_counts[rank.value() as usize];
            if count > 0 {
                groups.push((rank, count));
            }
        }

        // Sort by count (descending), then by rank (descending)
        groups.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

        Self { groups }
    }

    /// Returns the rank of a four-of-a-kind, if present.
    pub fn quad(&self) -> Option<Rank> {
        self.groups.iter().find(|(_, count)| *count == 4).map(|(rank, _)| *rank)
    }

    /// Returns the rank of a three-of-a-kind, if present.
    pub fn trips(&self) -> Option<Rank> {
        self.groups.iter().find(|(_, count)| *count == 3).map(|(rank, _)| *rank)
    }

    /// Returns all pair ranks, in descending order.
    pub fn pairs(&self) -> Vec<Rank> {
        self.groups.iter().filter(|(_, count)| *count == 2).map(|(rank, _)| *rank).collect()
    }

    /// Returns all singleton (kicker) ranks, in descending order.
    pub fn kickers(&self) -> Vec<Rank> {
        self.groups.iter().filter(|(_, count)| *count == 1).map(|(rank, _)| *rank).collect()
    }

    /// Returns true if the hand has both trips and a pair (full house).
    pub fn has_full_house(&self) -> bool {
        let has_trips = self.groups.iter().any(|(_, count)| *count == 3);
        let has_pair = self.groups.iter().any(|(_, count)| *count == 2);
        has_trips && has_pair
    }

    /// Returns the internal groups for debugging/testing.
    #[cfg(test)]
    pub fn groups(&self) -> &[(Rank, u8)] {
        &self.groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_counts(pairs: &[(u8, u8)]) -> [u8; 15] {
        let mut counts = [0u8; 15];
        for &(rank_val, count) in pairs {
            counts[rank_val as usize] = count;
        }
        counts
    }

    #[test]
    fn test_quad() {
        let counts = make_counts(&[(14, 4), (13, 1)]); // AAAAK
        let groups = RankGroups::from_counts(&counts);
        assert_eq!(groups.quad(), Some(Rank::Ace));
        assert_eq!(groups.trips(), None);
        assert_eq!(groups.pairs(), vec![]);
    }

    #[test]
    fn test_trips() {
        let counts = make_counts(&[(10, 3), (5, 1), (3, 1)]); // TTTT53
        let groups = RankGroups::from_counts(&counts);
        assert_eq!(groups.trips(), Some(Rank::Ten));
        assert_eq!(groups.quad(), None);
    }

    #[test]
    fn test_full_house() {
        let counts = make_counts(&[(14, 3), (13, 2)]); // AAAKK
        let groups = RankGroups::from_counts(&counts);
        assert!(groups.has_full_house());
        assert_eq!(groups.trips(), Some(Rank::Ace));
        assert_eq!(groups.pairs(), vec![Rank::King]);
    }

    #[test]
    fn test_two_pair() {
        let counts = make_counts(&[(14, 2), (13, 2), (10, 1)]); // AAKKT
        let groups = RankGroups::from_counts(&counts);
        let pairs = groups.pairs();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], Rank::Ace);
        assert_eq!(pairs[1], Rank::King);
        assert_eq!(groups.kickers(), vec![Rank::Ten]);
    }

    #[test]
    fn test_one_pair() {
        let counts = make_counts(&[(8, 2), (14, 1), (12, 1), (5, 1)]); // 88AQ5
        let groups = RankGroups::from_counts(&counts);
        assert_eq!(groups.pairs(), vec![Rank::Eight]);
        let kickers = groups.kickers();
        assert_eq!(kickers.len(), 3);
        assert_eq!(kickers[0], Rank::Ace);
        assert_eq!(kickers[1], Rank::Queen);
        assert_eq!(kickers[2], Rank::Five);
    }

    #[test]
    fn test_high_card() {
        let counts = make_counts(&[(14, 1), (10, 1), (7, 1), (5, 1), (2, 1)]); // AT752
        let groups = RankGroups::from_counts(&counts);
        assert_eq!(groups.quad(), None);
        assert_eq!(groups.trips(), None);
        assert_eq!(groups.pairs(), vec![]);
        assert_eq!(groups.kickers().len(), 5);
    }

    #[test]
    fn test_sorting() {
        let counts = make_counts(&[(5, 1), (14, 1), (10, 1)]); // A T 5
        let groups = RankGroups::from_counts(&counts);
        let ranks: Vec<Rank> = groups.groups().iter().map(|(r, _)| *r).collect();
        // Should be sorted by rank descending: A, T, 5
        assert_eq!(ranks, vec![Rank::Ace, Rank::Ten, Rank::Five]);
    }
}
