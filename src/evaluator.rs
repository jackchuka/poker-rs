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
    // Sort cards descending by rank (then suit) for stable output
    let mut sorted = *cards;
    sorted.sort_by(|a, b| b.rank().cmp(&a.rank()).then(b.suit().cmp(&a.suit())));

    // Rank counts
    let mut counts = [0u8; 15]; // 2..14 used
    let ranks =
        [sorted[0].rank(), sorted[1].rank(), sorted[2].rank(), sorted[3].rank(), sorted[4].rank()];
    for r in ranks.iter() {
        counts[*r as usize] += 1;
    }

    // Flush check
    let is_flush = sorted.iter().all(|c| c.suit() == sorted[0].suit());

    // Unique ranks ascending
    use std::collections::BTreeSet;
    let mut uniq_vals: Vec<u8> =
        ranks.iter().copied().map(|r| r as u8).collect::<BTreeSet<_>>().into_iter().collect();
    uniq_vals.sort_unstable();
    let is_wheel = uniq_vals == vec![2, 3, 4, 5, 14];
    let is_consecutive = uniq_vals.len() == 5 && uniq_vals.windows(2).all(|w| w[1] == w[0] + 1);
    let is_straight = is_wheel || is_consecutive;
    fn rank_from_val(v: u8) -> Rank {
        match v {
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            12 => Rank::Queen,
            13 => Rank::King,
            _ => Rank::Ace,
        }
    }
    let straight_top_rank: Rank = if is_straight {
        if is_wheel {
            Rank::Five
        } else {
            rank_from_val(*uniq_vals.last().unwrap())
        }
    } else {
        Rank::Two
    };

    // Straight Flush
    if is_flush && is_straight {
        let tiebreak = [straight_top_rank, Rank::Two, Rank::Two, Rank::Two, Rank::Two];
        let value = HandValue::from_parts(Category::StraightFlush, &tiebreak);
        return Evaluation { category: Category::StraightFlush, best_five: sorted, value };
    }

    // Build groups: (rank, count) sorted by (count desc, rank desc)
    let mut groups: Vec<(Rank, u8)> = (2u8..=14u8)
        .rev()
        .filter_map(|v| {
            let c = counts[v as usize];
            if c > 0 {
                Some((rank_from_val(v), c))
            } else {
                None
            }
        })
        .collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

    // Four of a kind
    if let Some(&(quad_rank, 4)) = groups.first() {
        let kicker = groups.iter().find(|(_, c)| *c == 1).map(|(r, _)| *r).unwrap_or(Rank::Two);
        let tiebreak = [quad_rank, kicker, Rank::Two, Rank::Two, Rank::Two];
        let value = HandValue::from_parts(Category::FourOfAKind, &tiebreak);
        return Evaluation { category: Category::FourOfAKind, best_five: sorted, value };
    }

    // Full House (3 + 2)
    if groups.len() >= 2 && groups[0].1 == 3 && groups[1].1 >= 2 {
        let trips = groups[0].0;
        let pair = groups[1].0;
        let tiebreak = [trips, pair, Rank::Two, Rank::Two, Rank::Two];
        let value = HandValue::from_parts(Category::FullHouse, &tiebreak);
        return Evaluation { category: Category::FullHouse, best_five: sorted, value };
    }

    // Flush
    if is_flush {
        let mut rdesc = ranks;
        rdesc.sort_by(|a, b| b.cmp(a));
        let value = HandValue::from_parts(Category::Flush, &rdesc);
        return Evaluation { category: Category::Flush, best_five: sorted, value };
    }

    // Straight
    if is_straight {
        let tiebreak = [straight_top_rank, Rank::Two, Rank::Two, Rank::Two, Rank::Two];
        let value = HandValue::from_parts(Category::Straight, &tiebreak);
        return Evaluation { category: Category::Straight, best_five: sorted, value };
    }

    // Three of a kind
    if let Some(&(trips_rank, 3)) = groups.first() {
        let mut kickers: Vec<Rank> =
            groups.iter().filter_map(|(r, c)| if *c == 1 { Some(*r) } else { None }).collect();
        kickers.sort_by(|a, b| b.cmp(a));
        let tiebreak = [trips_rank, kickers[0], kickers[1], Rank::Two, Rank::Two];
        let value = HandValue::from_parts(Category::ThreeOfAKind, &tiebreak);
        return Evaluation { category: Category::ThreeOfAKind, best_five: sorted, value };
    }

    // Two Pair
    let pairs: Vec<Rank> =
        groups.iter().filter_map(|(r, c)| if *c == 2 { Some(*r) } else { None }).collect();
    if pairs.len() >= 2 {
        let mut p = pairs.clone();
        p.sort_by(|a, b| b.cmp(a));
        let kicker = groups
            .iter()
            .find_map(|(r, c)| if *c == 1 { Some(*r) } else { None })
            .unwrap_or(Rank::Two);
        let tiebreak = [p[0], p[1], kicker, Rank::Two, Rank::Two];
        let value = HandValue::from_parts(Category::TwoPair, &tiebreak);
        return Evaluation { category: Category::TwoPair, best_five: sorted, value };
    }

    // One Pair
    if let Some(&(pair_rank, 2)) = groups.first() {
        let mut kickers: Vec<Rank> =
            groups.iter().filter_map(|(r, c)| if *c == 1 { Some(*r) } else { None }).collect();
        kickers.sort_by(|a, b| b.cmp(a));
        let tiebreak = [pair_rank, kickers[0], kickers[1], kickers[2], Rank::Two];
        let value = HandValue::from_parts(Category::Pair, &tiebreak);
        return Evaluation { category: Category::Pair, best_five: sorted, value };
    }

    // High Card
    let mut rdesc = ranks;
    rdesc.sort_by(|a, b| b.cmp(a));
    let value = HandValue::from_parts(Category::HighCard, &rdesc);
    Evaluation { category: Category::HighCard, best_five: sorted, value }
}

/// Evaluate seven cards (helper for Hold'em style 7-card evaluation).
/// Iterate all 21 five-card combinations from 7 and return the best by value.
pub fn evaluate_seven(cards: &[Card; 7]) -> Evaluation {
    let mut best: Option<Evaluation> = None;
    for i in 0..3 {
        for j in (i + 1)..4 {
            for k in (j + 1)..5 {
                for l in (k + 1)..6 {
                    for m in (l + 1)..7 {
                        let hand = [cards[i], cards[j], cards[k], cards[l], cards[m]];
                        let eval = evaluate_five(&hand);
                        if let Some(b) = best {
                            if eval > b {
                                best = Some(eval);
                            } else {
                                // keep current best
                            }
                        } else {
                            best = Some(eval);
                        }
                    }
                }
            }
        }
    }
    debug_assert!(best.is_some());
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
