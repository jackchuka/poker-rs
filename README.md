<div align="center">

# poker-rs

**A poker hand evaluator in Rust, with a terminal table to play it on.**

Rank and compare Texas Hold'em and Omaha hands with a small, validated library API,<br>
or sit down at a No-Limit Hold'em table against bots in a Ratatui TUI.

[![CI](https://github.com/jackchuka/poker-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jackchuka/poker-rs/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/jackchuka/poker-rs?sort=semver)](https://github.com/jackchuka/poker-rs/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

<img src="assets/demo.gif" alt="poker-rs TUI: configuring a table, playing a hand to showdown, then opening hand history and help" width="800">

[Installation](#installation) · [Quick Start](#quick-start) · [Controls](#controls) · [Library Usage](#library-usage) · [Development](#development)

</div>

---

Evaluating a poker hand is easy to get almost right: wheel straights, kicker order and duplicate cards are where hand-rolled code breaks. **poker-rs** keeps the evaluator deterministic and validated: duplicate cards, wrong board sizes and bad input come back as `Result` errors instead of panics, and every hand is ranked into a comparable value with full tie-breakers.

The same library drives a playable No-Limit Hold'em table in the terminal, including blinds, min-raise rules, all-ins and side pots, so the engine is exercised by real games, not just unit tests.

## Features

- **Hold'em evaluation** — `evaluate_holdem` picks the best five of seven cards; `compare_holdem` orders two hands on a shared board
- **Omaha evaluation** — `evaluate_omaha` / `compare_omaha` enforce the "exactly two hole + three board" rule
- **Raw evaluators** — `evaluate_five` and `evaluate_seven` for when you already have the cards
- **Card parsing** — `parse_cards("As, Kd 10c")` and `FromStr` for `Card`, `Rank`, `Suit`
- **No-Limit Hold'em engine** — blinds, button rotation, full-raise vs. short all-in rules, side pots, hand history
- **Bots** — four difficulty levels (Easy, Medium, Hard, Expert), settable per seat
- **Terminal UI** — 2 to 9 players, configurable stacks, blinds and bot delay, in-game help and history overlays
- **Tested** — property-based tests (`proptest`), integration tests and Criterion benchmarks

## Installation

### Homebrew (macOS, Linux)

```sh
brew install jackchuka/tap/poker-rs
```

### Binary

Download a prebuilt archive for macOS (arm64, amd64) or Linux (amd64) from the [releases page](https://github.com/jackchuka/poker-rs/releases).

### Cargo

poker-rs is not published on crates.io. Install the TUI from Git:

```sh
cargo install --git https://github.com/jackchuka/poker-rs
```

To use the library, add it as a Git dependency in `Cargo.toml`:

```toml
[dependencies]
poker-rs = { git = "https://github.com/jackchuka/poker-rs" }
```

### From Source

```sh
git clone https://github.com/jackchuka/poker-rs.git
cd poker-rs
cargo run --release --bin poker-rs
```

## Quick Start

1. **Start the TUI** in a real terminal (it exits with a message if stdout is not a TTY):

   ```sh
   poker-rs
   ```

2. **Configure the table** on the start screen with `Up`/`Down` and `+`/`-`, then press `Enter`. Defaults are 5 players, $1000 stacks, 5/10 blinds, Medium bots and a 500 ms bot delay.

3. **Play**: press `Space` to deal. You are seat P1; act with `F`, `C`, `B`, `R`, or `A` to type an exact amount. Press `?` at any time for help.

## Controls

| Context      | Key                     | Action                               |
| ------------ | ----------------------- | ------------------------------------ |
| Table        | `Space`                 | Deal / new hand                      |
|              | `F` / `C`               | Fold / call or check                 |
|              | `B` / `R`               | Bet minimum / raise minimum          |
|              | `A`                     | Open amount entry                    |
|              | `D`                     | Cycle the focused bot's difficulty   |
|              | `]` / `[` / `1`-`9`     | Focus next / previous / specific seat |
|              | `H`                     | Toggle hand history (`Up`/`Down` to scroll) |
|              | `?`                     | Toggle help                          |
|              | `M`                     | Open the menu                        |
| Amount entry | `0`-`9` / `Backspace`   | Edit the amount                      |
|              | `+` / `-`               | Adjust by one big blind              |
|              | `Enter` / `Esc`         | Submit / cancel                      |
| Menu         | `Up` / `Down`           | Move selection                       |
|              | `+` / `-`               | Adjust value                         |
|              | `Enter` / `Esc`         | Apply / cancel                       |
|              | `Q`                     | Quit                                 |

## Library Usage

Evaluate a Hold'em hand:

```rust
use poker_rs::cards::{Card, Rank, Suit};
use poker_rs::evaluator::{evaluate_holdem, Category};
use poker_rs::hand::{Board, HoleCards};

let hole = HoleCards::try_new(
    Card::new(Rank::Ace, Suit::Spades),
    Card::new(Rank::Ace, Suit::Hearts),
).unwrap();
let board = Board::try_new(vec![
    Card::new(Rank::King, Suit::Clubs),
    Card::new(Rank::Queen, Suit::Diamonds),
    Card::new(Rank::Jack, Suit::Hearts),
    Card::new(Rank::Three, Suit::Spades),
    Card::new(Rank::Two, Suit::Clubs),
]).unwrap();

let eval = evaluate_holdem(&hole, &board).unwrap();
assert_eq!(eval.category, Category::Pair);
```

Parse cards from text and compare two hands:

```rust
use poker_rs::cards::parse_cards;
use poker_rs::evaluator::compare_holdem;
use poker_rs::hand::{Board, HoleCards};
use std::cmp::Ordering;

let board = Board::try_new(parse_cards("Qc Jd 9h 3s 2c").unwrap()).unwrap();
let aces = HoleCards::from_slice(&parse_cards("As Ah").unwrap()).unwrap();
let kings = HoleCards::from_slice(&parse_cards("Ks Kh").unwrap()).unwrap();

assert_eq!(compare_holdem(&aces, &kings, &board).unwrap(), Ordering::Greater);
```

### Modules

| Module             | Contents                                                       |
| ------------------ | -------------------------------------------------------------- |
| `cards`            | `Card`, `Rank`, `Suit`, parsing and display                    |
| `hand`             | `HoleCards` and `Board` with validation                        |
| `evaluator`        | `evaluate_holdem`, `evaluate_five`, `evaluate_seven`, `compare_holdem`, `Category`, `Evaluation` |
| `variants::omaha`  | `OmahaHoleCards`, `evaluate_omaha`, `compare_omaha`            |
| `deck`             | Shuffling and dealing with a seedable RNG                      |
| `game` / `engine`  | No-Limit Hold'em game state, betting rules, side pots, history |
| `agents`           | Human and bot agents, `Difficulty`, `BotProfile`               |
| `tui`              | Ratatui views, state and input (used by the `poker-rs` binary) |

API docs: `cargo doc --open`.

## Development

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
cargo bench --features bench   # Criterion benchmarks, see doc/benchmarks.md
```

Regenerate the demo with `vhs assets/demo.tape` from the repository root.

## License

[MIT](LICENSE)
