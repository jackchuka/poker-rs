//! poker-rs: Poker evaluation library
//!
//! Goals:
//! - Deterministic, fast evaluation for Texas Hold'em (initially)
//! - Small, well-documented public API
//! - No panics for invalid input; use `Result` for recoverable errors
//!
//! ## Quick start: evaluate a Hold'em hand
//! ```
//! use poker_rs::cards::{Card, Rank, Suit};
//! use poker_rs::evaluator::{evaluate_holdem, Category};
//! use poker_rs::hand::{Board, HoleCards};
//!
//! let hole = HoleCards::try_new(
//!     Card::new(Rank::Ace, Suit::Spades),
//!     Card::new(Rank::Ace, Suit::Hearts),
//! ).unwrap();
//! let board = Board::try_new(vec![
//!     Card::new(Rank::King, Suit::Clubs),
//!     Card::new(Rank::Queen, Suit::Diamonds),
//!     Card::new(Rank::Jack, Suit::Hearts),
//!     Card::new(Rank::Three, Suit::Spades),
//!     Card::new(Rank::Two, Suit::Clubs),
//! ]).unwrap();
//!
//! let eval = evaluate_holdem(&hole, &board).unwrap();
//! assert_eq!(eval.category, Category::Pair);
//! ```
//!
//! ## TUI
//! Run the interactive TUI with:
//! ```sh
//! cargo run --bin poker-rs
//! ```

pub mod agents;
pub mod cards;
pub mod deck;
pub mod engine;
pub mod evaluator;
pub mod game;
pub mod hand;
pub mod tui;
pub mod variants;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
