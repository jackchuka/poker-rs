use std::fmt;
use std::str::FromStr;

/// Card ranks from Two (low) to Ace (high).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    pub const fn value(self) -> u8 {
        self as u8
    }

    pub const fn to_char(self) -> char {
        match self {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RankParseError {
    #[error("invalid rank: '{0}'")]
    Invalid(String),
}

impl FromStr for Rank {
    type Err = RankParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        let upper = t.to_ascii_uppercase();
        let r = match upper.as_str() {
            "2" => Rank::Two,
            "3" => Rank::Three,
            "4" => Rank::Four,
            "5" => Rank::Five,
            "6" => Rank::Six,
            "7" => Rank::Seven,
            "8" => Rank::Eight,
            "9" => Rank::Nine,
            "10" | "T" => Rank::Ten,
            "J" => Rank::Jack,
            "Q" => Rank::Queen,
            "K" => Rank::King,
            "A" => Rank::Ace,
            _ => return Err(RankParseError::Invalid(s.to_string())),
        };
        Ok(r)
    }
}

impl TryFrom<char> for Rank {
    type Error = RankParseError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        let up = c.to_ascii_uppercase();
        match up {
            '2' => Ok(Rank::Two),
            '3' => Ok(Rank::Three),
            '4' => Ok(Rank::Four),
            '5' => Ok(Rank::Five),
            '6' => Ok(Rank::Six),
            '7' => Ok(Rank::Seven),
            '8' => Ok(Rank::Eight),
            '9' => Ok(Rank::Nine),
            'T' => Ok(Rank::Ten),
            'J' => Ok(Rank::Jack),
            'Q' => Ok(Rank::Queen),
            'K' => Ok(Rank::King),
            'A' => Ok(Rank::Ace),
            _ => Err(RankParseError::Invalid(c.to_string())),
        }
    }
}

/// Four suits; order has no hand-strength meaning but is fixed for ordering: C < D < H < S.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    pub const fn to_char(self) -> char {
        match self {
            Suit::Clubs => 'c',
            Suit::Diamonds => 'd',
            Suit::Hearts => 'h',
            Suit::Spades => 's',
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SuitParseError {
    #[error("invalid suit: '{0}'")]
    Invalid(String),
}

impl FromStr for Suit {
    type Err = SuitParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t.len() == 1 {
            return Suit::try_from(t.chars().next().unwrap());
        }
        match t.to_ascii_lowercase().as_str() {
            "clubs" => Ok(Suit::Clubs),
            "diamonds" => Ok(Suit::Diamonds),
            "hearts" => Ok(Suit::Hearts),
            "spades" => Ok(Suit::Spades),
            _ => Err(SuitParseError::Invalid(s.to_string())),
        }
    }
}

impl TryFrom<char> for Suit {
    type Error = SuitParseError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'c' => Ok(Suit::Clubs),
            'd' => Ok(Suit::Diamonds),
            'h' => Ok(Suit::Hearts),
            's' => Ok(Suit::Spades),
            _ => Err(SuitParseError::Invalid(c.to_string())),
        }
    }
}

/// A playing card: rank + suit.
///
/// ```
/// use poker_rs::cards::{Card, Rank, Suit};
///
/// let card = Card::new(Rank::Ace, Suit::Spades);
/// assert_eq!(card.to_string(), "As");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    pub const fn rank(self) -> Rank {
        self.rank
    }
    pub const fn suit(self) -> Suit {
        self.suit
    }

    pub const fn to_tuple(self) -> (Rank, Suit) {
        (self.rank, self.suit)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CardParseError {
    #[error("invalid card: '{0}'")]
    Invalid(String),
    #[error(transparent)]
    Rank(#[from] RankParseError),
    #[error(transparent)]
    Suit(#[from] SuitParseError),
}

impl FromStr for Card {
    type Err = CardParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t.len() < 2 {
            return Err(CardParseError::Invalid(s.to_string()));
        }

        // rank is first char or "10"; suit is last char
        let (rank_str, suit_ch) = if t.len() == 2 {
            (&t[..1], t.chars().nth(1).unwrap())
        } else if t.len() == 3 && &t[..2].to_ascii_uppercase() == "10" {
            (&t[..2], t.chars().nth(2).unwrap())
        } else {
            // Also support formats with no ambiguity: last char suit
            (&t[..t.len() - 1], t.chars().last().unwrap())
        };

        let rank = Rank::from_str(rank_str)?;
        let suit = Suit::try_from(suit_ch)?;
        Ok(Card::new(rank, suit))
    }
}

/// Parse multiple cards separated by whitespace or commas.
///
/// ```
/// use poker_rs::cards::{parse_cards, Card, Rank, Suit};
///
/// let cards = parse_cards("As, Kd 10c").unwrap();
/// assert_eq!(cards[0], Card::new(Rank::Ace, Suit::Spades));
/// assert_eq!(cards[1], Card::new(Rank::King, Suit::Diamonds));
/// assert_eq!(cards[2], Card::new(Rank::Ten, Suit::Clubs));
/// ```
pub fn parse_cards(input: &str) -> Result<Vec<Card>, CardParseError> {
    input
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(Card::from_str)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_display_and_from_str() {
        assert_eq!(Rank::Ace.to_string(), "A");
        assert_eq!(Rank::from_str("T").unwrap(), Rank::Ten);
        assert_eq!(Rank::from_str("10").unwrap(), Rank::Ten);
        assert!(Rank::from_str("1").is_err());
    }

    #[test]
    fn suit_display_and_from_str() {
        assert_eq!(Suit::Spades.to_string(), "s");
        assert_eq!(Suit::from_str("s").unwrap(), Suit::Spades);
        assert_eq!(Suit::from_str("Hearts").unwrap(), Suit::Hearts);
        assert!(Suit::from_str("x").is_err());
    }

    #[test]
    fn card_display_and_from_str() {
        let a = Card::new(Rank::Ace, Suit::Spades);
        assert_eq!(a.to_string(), "As");
        assert_eq!(Card::from_str("As").unwrap(), a);
        assert_eq!(Card::from_str("10d").unwrap(), Card::new(Rank::Ten, Suit::Diamonds));
        assert_eq!(Card::from_str("ah").unwrap(), Card::new(Rank::Ace, Suit::Hearts));
    }

    #[test]
    fn ordering_is_rank_then_suit() {
        let as_ = Card::new(Rank::Ace, Suit::Spades);
        let ah = Card::new(Rank::Ace, Suit::Hearts);
        let kd = Card::new(Rank::King, Suit::Diamonds);
        assert!(as_ > ah);
        assert!(ah > kd);
    }

    #[test]
    fn parse_many_cards() {
        let xs = parse_cards("As, Kd 10c").unwrap();
        assert_eq!(xs.len(), 3);
        assert_eq!(xs[0], Card::new(Rank::Ace, Suit::Spades));
        assert_eq!(xs[1], Card::new(Rank::King, Suit::Diamonds));
        assert_eq!(xs[2], Card::new(Rank::Ten, Suit::Clubs));
    }
}
