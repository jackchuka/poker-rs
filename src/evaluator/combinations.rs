/// Iterator for C(4,2) = 6 combinations (choosing 2 from 4 hole cards in Omaha).
pub struct Combinations4Choose2 {
    indices: [usize; 2],
    done: bool,
}

impl Combinations4Choose2 {
    pub fn new() -> Self {
        Self { indices: [0, 1], done: false }
    }
}

impl Default for Combinations4Choose2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Combinations4Choose2 {
    type Item = [usize; 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let result = self.indices;

        // Try to increment the second index
        if self.indices[1] < 3 {
            self.indices[1] += 1;
        } else if self.indices[0] < 2 {
            // Move to next first index and reset second
            self.indices[0] += 1;
            self.indices[1] = self.indices[0] + 1;
        } else {
            // Exhausted all combinations
            self.done = true;
        }

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            (1, Some(6))
        }
    }
}

/// Iterator for C(5,3) = 10 combinations (choosing 3 from 5 board cards in Omaha).
pub struct Combinations5Choose3 {
    indices: [usize; 3],
    done: bool,
}

impl Combinations5Choose3 {
    pub fn new() -> Self {
        Self { indices: [0, 1, 2], done: false }
    }
}

impl Default for Combinations5Choose3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Combinations5Choose3 {
    type Item = [usize; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let result = self.indices;

        // Find rightmost index that can be incremented
        let mut i = 2;
        loop {
            if self.indices[i] < 5 - (3 - i) {
                self.indices[i] += 1;
                // Reset all indices to the right
                for j in (i + 1)..3 {
                    self.indices[j] = self.indices[j - 1] + 1;
                }
                break;
            }

            if i == 0 {
                self.done = true;
                break;
            }
            i -= 1;
        }

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            (1, Some(10))
        }
    }
}

/// Iterator that generates all C(7,5) = 21 combinations of choosing 5 indices from 7.
///
/// This replaces the 5-level nested loop structure with a clean iterator pattern.
/// The combinations are generated in lexicographic order.
pub struct Combinations7Choose5 {
    indices: [usize; 5],
    done: bool,
}

impl Combinations7Choose5 {
    /// Create a new iterator for 5-combinations from 7 elements.
    pub fn new() -> Self {
        Self {
            indices: [0, 1, 2, 3, 4], // Start with first combination
            done: false,
        }
    }
}

impl Default for Combinations7Choose5 {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Combinations7Choose5 {
    type Item = [usize; 5];

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let result = self.indices;

        // Find the rightmost index that can be incremented
        let mut i = 4;
        loop {
            // Try to increment index i
            if self.indices[i] < 7 - (5 - i) {
                self.indices[i] += 1;

                // Reset all indices to the right
                for j in (i + 1)..5 {
                    self.indices[j] = self.indices[j - 1] + 1;
                }
                break;
            }

            // If we can't increment, move left
            if i == 0 {
                // All combinations exhausted
                self.done = true;
                break;
            }
            i -= 1;
        }

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            // C(7,5) = 21 combinations
            // We could track how many we've yielded, but for simplicity just give a range
            (1, Some(21))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4choose2_generates_6_combinations() {
        let combos: Vec<[usize; 2]> = Combinations4Choose2::new().collect();
        assert_eq!(combos.len(), 6);
    }

    #[test]
    fn test_4choose2_all_valid() {
        for combo in Combinations4Choose2::new() {
            assert!(combo.iter().all(|&i| i < 4));
            assert!(combo[1] > combo[0]);
        }
    }

    #[test]
    fn test_4choose2_specific() {
        let combos: Vec<[usize; 2]> = Combinations4Choose2::new().collect();
        assert_eq!(combos[0], [0, 1]);
        assert_eq!(combos[1], [0, 2]);
        assert_eq!(combos[2], [0, 3]);
        assert_eq!(combos[3], [1, 2]);
        assert_eq!(combos[4], [1, 3]);
        assert_eq!(combos[5], [2, 3]);
    }

    #[test]
    fn test_4choose2_no_duplicates() {
        let combos: Vec<[usize; 2]> = Combinations4Choose2::new().collect();
        let mut seen = std::collections::HashSet::new();
        for combo in combos {
            assert!(seen.insert(combo), "Duplicate: {combo:?}");
        }
    }

    #[test]
    fn test_5choose3_generates_10_combinations() {
        let combos: Vec<[usize; 3]> = Combinations5Choose3::new().collect();
        assert_eq!(combos.len(), 10);
    }

    #[test]
    fn test_5choose3_all_valid() {
        for combo in Combinations5Choose3::new() {
            assert!(combo.iter().all(|&i| i < 5));
            assert!(combo[1] > combo[0]);
            assert!(combo[2] > combo[1]);
        }
    }

    #[test]
    fn test_5choose3_specific() {
        let combos: Vec<[usize; 3]> = Combinations5Choose3::new().collect();
        assert_eq!(combos[0], [0, 1, 2]);
        assert_eq!(combos[1], [0, 1, 3]);
        assert_eq!(combos[2], [0, 1, 4]);
        assert_eq!(combos[3], [0, 2, 3]);
        assert_eq!(combos[4], [0, 2, 4]);
        assert_eq!(combos[5], [0, 3, 4]);
        assert_eq!(combos[6], [1, 2, 3]);
        assert_eq!(combos[7], [1, 2, 4]);
        assert_eq!(combos[8], [1, 3, 4]);
        assert_eq!(combos[9], [2, 3, 4]);
    }

    #[test]
    fn test_5choose3_no_duplicates() {
        let combos: Vec<[usize; 3]> = Combinations5Choose3::new().collect();
        let mut seen = std::collections::HashSet::new();
        for combo in combos {
            assert!(seen.insert(combo), "Duplicate: {combo:?}");
        }
    }

    #[test]
    fn test_generates_21_combinations() {
        let combos: Vec<[usize; 5]> = Combinations7Choose5::new().collect();
        assert_eq!(combos.len(), 21);
    }

    #[test]
    fn test_all_combinations_valid() {
        for combo in Combinations7Choose5::new() {
            // All indices should be < 7
            assert!(combo.iter().all(|&i| i < 7));

            // All indices should be in ascending order
            for i in 1..5 {
                assert!(combo[i] > combo[i - 1]);
            }
        }
    }

    #[test]
    fn test_first_combination() {
        let mut iter = Combinations7Choose5::new();
        assert_eq!(iter.next(), Some([0, 1, 2, 3, 4]));
    }

    #[test]
    fn test_last_combination() {
        let combos: Vec<[usize; 5]> = Combinations7Choose5::new().collect();
        assert_eq!(combos.last(), Some(&[2, 3, 4, 5, 6]));
    }

    #[test]
    fn test_no_duplicates() {
        let combos: Vec<[usize; 5]> = Combinations7Choose5::new().collect();
        let mut seen = std::collections::HashSet::new();

        for combo in combos {
            assert!(seen.insert(combo), "Duplicate combination found: {combo:?}");
        }
    }

    #[test]
    fn test_specific_combinations() {
        let combos: Vec<[usize; 5]> = Combinations7Choose5::new().collect();

        // Check a few known combinations
        assert!(combos.contains(&[0, 1, 2, 3, 4]));
        assert!(combos.contains(&[0, 1, 2, 3, 5]));
        assert!(combos.contains(&[0, 1, 2, 3, 6]));
        assert!(combos.contains(&[0, 1, 2, 4, 5]));
        assert!(combos.contains(&[2, 3, 4, 5, 6]));
    }

    #[test]
    fn test_lexicographic_order() {
        let combos: Vec<[usize; 5]> = Combinations7Choose5::new().collect();

        // Verify lexicographic ordering
        for i in 1..combos.len() {
            let prev = combos[i - 1];
            let curr = combos[i];

            // Find first position where they differ
            for j in 0..5 {
                if prev[j] != curr[j] {
                    assert!(
                        prev[j] < curr[j],
                        "Not in lexicographic order: {prev:?} should come before {curr:?}"
                    );
                    break;
                }
            }
        }
    }

    #[test]
    fn test_iterator_exhausts() {
        let mut iter = Combinations7Choose5::new();

        // Consume all 21 combinations
        for _ in 0..21 {
            assert!(iter.next().is_some());
        }

        // Should be exhausted now
        assert!(iter.next().is_none());
        assert!(iter.next().is_none()); // Still none
    }
}
