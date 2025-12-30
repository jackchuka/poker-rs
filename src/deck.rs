use crate::cards::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// A standard 52-card deck.
#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// ```
    /// use poker_rs::deck::Deck;
    ///
    /// let deck = Deck::standard();
    /// assert_eq!(deck.len(), 52);
    /// ```
    pub fn standard() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &s in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for &r in &[
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
            ] {
                cards.push(Card::new(r, s));
            }
        }
        Self { cards }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Shuffle using a seeded RNG for reproducibility.
    pub fn shuffle_seeded(&mut self, seed: u64) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        self.cards.shuffle(&mut rng);
    }

    /// Shuffle using the provided RNG implementing Rng.
    pub fn shuffle_with<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
    }

    /// Draw one card from the top of the deck.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Draw `n` cards from the top of the deck.
    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        (0..n).filter_map(|_| self.draw()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_deck_has_52_cards() {
        let d = Deck::standard();
        assert_eq!(d.len(), 52);
    }

    #[test]
    fn seeded_shuffle_is_reproducible() {
        let mut d1 = Deck::standard();
        let mut d2 = Deck::standard();
        d1.shuffle_seeded(42);
        d2.shuffle_seeded(42);
        assert_eq!(d1.cards, d2.cards);
    }

    #[test]
    fn draw_reduces_length_and_returns_cards() {
        let mut d = Deck::standard();
        d.shuffle_seeded(7);
        let c1 = d.draw().unwrap();
        let c2 = d.draw().unwrap();
        assert_ne!(c1, c2);
        assert_eq!(d.len(), 50);
        let hand = d.draw_n(5);
        assert_eq!(hand.len(), 5);
        assert_eq!(d.len(), 45);
    }
}
