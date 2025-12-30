use crate::cards::Card;
use crate::engine::GameEngine;
use crate::evaluator::{evaluate_five, evaluate_seven, Evaluation};
use crate::hand::HoleCards;
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use std::time::{Duration, Instant};

use super::{Action, AgentKind, PlayerAgent};

/// Difficulty tiers for bot play style and mistake rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

/// Configuration for a bot's play style and randomness.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BotProfile {
    pub difficulty: Difficulty,
    pub tightness: f64,
    pub aggression: f64,
    pub bluff: f64,
    pub tilt: f64,
    pub curiosity: f64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub rng_seed: Option<u64>,
}

impl BotProfile {
    /// Create a profile with tuned defaults for a difficulty tier.
    pub fn for_difficulty(difficulty: Difficulty) -> Self {
        let (tightness, aggression, bluff, tilt, curiosity) = match difficulty {
            Difficulty::Easy => (0.3, 0.18, 0.03, 0.3, 0.4),
            Difficulty::Medium => (0.5, 0.35, 0.05, 0.15, 0.2),
            Difficulty::Hard => (0.62, 0.48, 0.08, 0.08, 0.12),
            Difficulty::Expert => (0.72, 0.6, 0.12, 0.05, 0.1),
        };
        Self {
            difficulty,
            tightness,
            aggression,
            bluff,
            tilt,
            curiosity,
            min_delay_ms: 0,
            max_delay_ms: 0,
            rng_seed: None,
        }
    }

    /// Set a deterministic RNG seed for reproducible decisions.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng_seed = Some(seed);
        self
    }
}

impl Default for BotProfile {
    fn default() -> Self {
        Self::for_difficulty(Difficulty::Medium)
    }
}

/// Back-compat name for older callers.
pub type BotConfig = BotProfile;

#[derive(Debug)]
struct BotState {
    rng: StdRng,
}

impl BotState {
    fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(v) => StdRng::seed_from_u64(v),
            None => {
                let mut seed = [0u8; 32];
                rand::rng().fill_bytes(&mut seed);
                StdRng::from_seed(seed)
            }
        };
        Self { rng }
    }
}

#[derive(Debug, Clone)]
struct BotDecision {
    action: Action,
    #[allow(dead_code)]
    confidence: f64,
    #[allow(dead_code)]
    reason: &'static str,
}

struct BotPolicy;

impl BotPolicy {
    fn decide(ctx: &BotContext<'_>, profile: &BotProfile, state: &mut BotState) -> BotDecision {
        let position = position_bucket(ctx.seat, ctx.dealer, ctx.num_players);
        let strength = estimate_strength(ctx.hole, ctx.board, position);
        let pot_odds = if ctx.to_call == 0 {
            0.0
        } else {
            ctx.to_call as f64 / (ctx.pot + ctx.to_call) as f64
        };
        let position_factor = position_factor(ctx.seat, ctx.dealer, ctx.num_players);
        let (mistake_rate, diff_bias) = difficulty_modifiers(profile.difficulty);
        let tilt_bias = state.rng.random_range(-1.0..=1.0) * profile.tilt * 0.05;
        let tightness = (profile.tightness + diff_bias - position_factor).clamp(0.05, 0.95);
        let aggression =
            (profile.aggression + diff_bias + position_factor + tilt_bias).clamp(0.05, 0.95);
        let bluff = (profile.bluff + diff_bias * 0.5).clamp(0.0, 0.5);
        let curiosity = profile.curiosity.clamp(0.0, 0.6);

        let noise = state.rng.random_range(-1.0..=1.0) * mistake_rate * 0.18;
        let adjusted = (strength + noise).clamp(0.0, 1.0);

        let mut fold_threshold = 0.35 + tightness * 0.3;
        fold_threshold = (fold_threshold - pot_odds * 0.25).clamp(0.1, 0.9);
        let raise_threshold = (0.68 - aggression * 0.25).clamp(0.15, 0.9);

        let params = DecisionParams {
            adjusted,
            fold_threshold,
            raise_threshold,
            aggression,
            bluff,
            curiosity,
        };

        if ctx.to_call > 0 {
            return decide_facing_bet(ctx, state, params);
        }

        if ctx.current_bet > 0 {
            return decide_when_checked(
                ctx,
                state,
                params,
                CheckedActionConfig {
                    choose_target: choose_raise_target,
                    action: Action::RaiseTo,
                    value_reason: "value_raise",
                    bluff_reason: "bluff_raise",
                },
            );
        }

        decide_when_checked(
            ctx,
            state,
            params,
            CheckedActionConfig {
                choose_target: choose_bet_target,
                action: Action::Bet,
                value_reason: "value_bet",
                bluff_reason: "bluff",
            },
        )
    }
}

fn decide_facing_bet(
    ctx: &BotContext<'_>,
    state: &mut BotState,
    params: DecisionParams,
) -> BotDecision {
    if params.adjusted < params.fold_threshold && state.rng.random::<f64>() > params.curiosity * 0.3
    {
        return BotDecision {
            action: Action::Fold,
            confidence: 1.0 - params.adjusted,
            reason: "fold",
        };
    }
    if params.adjusted > params.raise_threshold && state.rng.random::<f64>() < params.aggression {
        let target = choose_raise_target(ctx, params.aggression, params.adjusted);
        return BotDecision {
            action: Action::RaiseTo(target),
            confidence: params.adjusted,
            reason: "value_raise",
        };
    }
    BotDecision {
        action: Action::CheckCall,
        confidence: 1.0 - (params.fold_threshold - params.adjusted).abs(),
        reason: "call",
    }
}

fn decide_when_checked(
    ctx: &BotContext<'_>,
    state: &mut BotState,
    params: DecisionParams,
    cfg: CheckedActionConfig,
) -> BotDecision {
    if params.adjusted > params.raise_threshold && state.rng.random::<f64>() < params.aggression {
        let target = (cfg.choose_target)(ctx, params.aggression, params.adjusted);
        return BotDecision {
            action: (cfg.action)(target),
            confidence: params.adjusted,
            reason: cfg.value_reason,
        };
    }
    if params.adjusted < params.fold_threshold && state.rng.random::<f64>() < params.bluff {
        let target = (cfg.choose_target)(ctx, params.aggression, params.adjusted);
        return BotDecision {
            action: (cfg.action)(target),
            confidence: 1.0 - params.adjusted,
            reason: cfg.bluff_reason,
        };
    }
    BotDecision { action: Action::CheckCall, confidence: 0.5, reason: "check" }
}

#[derive(Clone, Copy)]
struct DecisionParams {
    adjusted: f64,
    fold_threshold: f64,
    raise_threshold: f64,
    aggression: f64,
    bluff: f64,
    curiosity: f64,
}

#[derive(Clone, Copy)]
struct CheckedActionConfig {
    choose_target: fn(&BotContext<'_>, f64, f64) -> u64,
    action: fn(u64) -> Action,
    value_reason: &'static str,
    bluff_reason: &'static str,
}

struct BotContext<'a> {
    seat: usize,
    dealer: usize,
    num_players: usize,
    to_call: u64,
    pot: u64,
    current_bet: u64,
    min_raise: u64,
    stack: u64,
    bet: u64,
    hole: &'a HoleCards,
    board: &'a crate::hand::Board,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionBucket {
    HeadsUp,
    Button,
    SmallBlind,
    BigBlind,
    Early,
    Middle,
    Late,
}

/// A flexible bot agent with adjustable profile and difficulty tiers.
pub struct BotAgent {
    profile: BotProfile,
    state: BotState,
    next_action_at: Option<Instant>,
}

impl BotAgent {
    pub fn new(profile: BotProfile) -> Self {
        let state = BotState::new(profile.rng_seed);
        Self { profile, state, next_action_at: None }
    }
}

impl PlayerAgent for BotAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Bot
    }
    fn on_turn(
        &mut self,
        engine: &mut dyn GameEngine,
        seat: usize,
    ) -> Result<bool, crate::game::ActionError> {
        if matches!(engine.street(), crate::game::Street::Showdown) {
            return Ok(false);
        }
        if engine.current() != seat {
            return Ok(false);
        }
        let now = Instant::now();
        let delay = choose_delay_ms(&self.profile, &mut self.state);
        if delay > 0 {
            match self.next_action_at {
                None => {
                    self.next_action_at = Some(now + Duration::from_millis(delay));
                    return Ok(false);
                }
                Some(next) if now < next => {
                    return Ok(false);
                }
                Some(_) => {}
            }
        }
        self.next_action_at = None;

        // Heads-up preflop: avoid folding to the blind.
        if matches!(engine.street(), crate::game::Street::Preflop)
            && engine.num_players() == 2
            && engine.current_bet() == engine.min_raise()
            && engine.to_call(seat) > 0
        {
            return engine.action_check_call().map(|_| true);
        }

        let hole = match engine.hole_cards(seat) {
            Some(h) => h,
            None => return Ok(false),
        };
        let ctx = BotContext {
            seat,
            dealer: engine.dealer(),
            num_players: engine.num_players(),
            to_call: engine.to_call(seat),
            pot: engine.pot(),
            current_bet: engine.current_bet(),
            min_raise: engine.min_raise(),
            stack: engine.stack(seat),
            bet: engine.bet(seat),
            hole: &hole,
            board: engine.board(),
        };

        let decision = BotPolicy::decide(&ctx, &self.profile, &mut self.state);
        let result = match decision.action {
            Action::Fold => engine.action_fold(),
            Action::CheckCall => engine.action_check_call(),
            Action::BetMin => engine.action_bet_min(),
            Action::RaiseMin => engine.action_raise_min(),
            Action::Bet(amount) => engine.action_bet(amount),
            Action::RaiseTo(amount) => engine.action_raise_to(amount),
        };
        result.map(|_| true)
    }
}

fn choose_delay_ms(profile: &BotProfile, state: &mut BotState) -> u64 {
    let min = profile.min_delay_ms;
    let max = profile.max_delay_ms.max(min);
    if max == min {
        min
    } else {
        state.rng.random_range(min..=max)
    }
}

fn position_bucket(seat: usize, dealer: usize, num_players: usize) -> PositionBucket {
    if num_players <= 2 {
        return PositionBucket::HeadsUp;
    }
    let dist = (seat + num_players - dealer) % num_players;
    if dist == 0 {
        return PositionBucket::Button;
    }
    if dist == 1 {
        return PositionBucket::SmallBlind;
    }
    if dist == 2 {
        return PositionBucket::BigBlind;
    }
    let active = num_players.saturating_sub(3);
    if active <= 1 {
        return PositionBucket::Late;
    }
    let rel = dist.saturating_sub(3);
    if rel < active / 3 {
        PositionBucket::Early
    } else if rel < (2 * active) / 3 {
        PositionBucket::Middle
    } else {
        PositionBucket::Late
    }
}

fn difficulty_modifiers(difficulty: Difficulty) -> (f64, f64) {
    match difficulty {
        Difficulty::Easy => (0.28, -0.1),
        Difficulty::Medium => (0.14, 0.0),
        Difficulty::Hard => (0.08, 0.05),
        Difficulty::Expert => (0.04, 0.09),
    }
}

fn position_factor(seat: usize, dealer: usize, num_players: usize) -> f64 {
    if num_players <= 2 {
        return 0.0;
    }
    let dist = (seat + num_players - dealer) % num_players;
    let frac = dist as f64 / num_players as f64;
    if frac <= 0.2 {
        0.08
    } else if frac >= 0.7 {
        -0.08
    } else {
        0.0
    }
}

fn choose_bet_target(ctx: &BotContext<'_>, aggression: f64, strength: f64) -> u64 {
    let min_bet = ctx.min_raise.max(1);
    let max_total = ctx.bet + ctx.stack;
    if max_total <= min_bet {
        return max_total;
    }
    if strength > 0.85 && ctx.stack <= ctx.pot.saturating_add(ctx.to_call) {
        return max_total;
    }
    let base_factor = if strength > 0.8 {
        0.9
    } else if strength > 0.6 {
        0.6
    } else {
        0.33
    };
    let scale = 0.8 + aggression * 0.4;
    let size = if ctx.pot == 0 {
        min_bet
    } else {
        ((ctx.pot as f64) * base_factor * scale).round() as u64
    };
    size.max(min_bet).min(max_total)
}

fn choose_raise_target(ctx: &BotContext<'_>, aggression: f64, strength: f64) -> u64 {
    let min_raise = ctx.min_raise.max(1);
    let max_total = ctx.bet + ctx.stack;
    if max_total <= ctx.current_bet + 1 {
        return max_total;
    }
    if strength > 0.88 && ctx.stack <= ctx.pot.saturating_add(ctx.to_call) {
        return max_total;
    }
    let base_factor = if strength > 0.85 {
        1.0
    } else if strength > 0.65 {
        0.7
    } else {
        0.5
    };
    let scale = 0.9 + aggression * 0.3;
    let raise = ((ctx.pot.max(ctx.current_bet) as f64) * base_factor * scale).round() as u64;
    let target = ctx.current_bet + min_raise.max(raise);
    target.min(max_total)
}

fn estimate_strength(
    hole: &HoleCards,
    board: &crate::hand::Board,
    position: PositionBucket,
) -> f64 {
    let board_cards = board.as_slice();
    if board_cards.is_empty() {
        return preflop_strength_with_position(hole, position);
    }
    let mut cards = Vec::with_capacity(2 + board_cards.len());
    cards.push(hole.first());
    cards.push(hole.second());
    cards.extend_from_slice(board_cards);

    if cards.len() >= 5 {
        if let Some(eval) = best_eval(&cards) {
            let base = eval.category.ordinal() as f64 / 8.0;
            let high = eval.best_five[0].rank().value() as f64 / 14.0;
            let mut strength = base * 0.85 + high * 0.15;
            if board_cards.len() < 5 {
                strength = (strength + draw_bonus(hole, board_cards, &cards)).min(1.0);
            }
            let texture = board_texture(board_cards);
            let category_weight = eval.category.ordinal() as f64 / 8.0;
            let texture_penalty = texture * (0.12 * (1.0 - category_weight));
            return (strength - texture_penalty).clamp(0.0, 1.0);
        }
    }
    preflop_strength_with_position(hole, position)
}

fn best_eval(cards: &[Card]) -> Option<Evaluation> {
    match cards.len() {
        5 => {
            let arr: [Card; 5] = cards.try_into().ok()?;
            Some(evaluate_five(&arr))
        }
        6 => {
            let mut best: Option<Evaluation> = None;
            for skip in 0..6 {
                let mut five = [cards[0]; 5];
                let mut idx = 0;
                for (i, c) in cards.iter().enumerate() {
                    if i == skip {
                        continue;
                    }
                    five[idx] = *c;
                    idx += 1;
                }
                let ev = evaluate_five(&five);
                if best.map_or(true, |b| ev > b) {
                    best = Some(ev);
                }
            }
            best
        }
        7 => {
            let arr: [Card; 7] = cards.try_into().ok()?;
            Some(evaluate_seven(&arr))
        }
        _ => None,
    }
}

fn preflop_strength(hole: &HoleCards) -> f64 {
    let a = hole.first().rank().value() as i32;
    let b = hole.second().rank().value() as i32;
    let high = a.max(b) as f64;
    let low = a.min(b) as f64;
    let pair = a == b;
    let suited = hole.first().suit() == hole.second().suit();
    let gap = (high - low) as i32;

    let mut score = (high / 14.0) * 0.5 + (low / 14.0) * 0.1;
    if pair {
        score += 0.3 + (high / 14.0) * 0.1;
    }
    if suited {
        score += 0.05;
    }
    if gap == 1 {
        score += 0.05;
    } else if gap == 2 {
        score += 0.02;
    } else if gap > 4 {
        score -= 0.05;
    }
    score.clamp(0.0, 1.0)
}

fn preflop_strength_with_position(hole: &HoleCards, position: PositionBucket) -> f64 {
    let base = preflop_strength(hole);
    let in_range = preflop_in_range(hole, position);
    let (bonus, penalty) = match position {
        PositionBucket::HeadsUp => (0.12, 0.04),
        PositionBucket::Button | PositionBucket::Late => (0.1, 0.06),
        PositionBucket::Middle => (0.08, 0.08),
        PositionBucket::Early => (0.06, 0.1),
        PositionBucket::SmallBlind | PositionBucket::BigBlind => (0.07, 0.07),
    };
    if in_range {
        (base + bonus).min(1.0)
    } else {
        (base - penalty).max(0.0)
    }
}

fn preflop_in_range(hole: &HoleCards, position: PositionBucket) -> bool {
    let a = hole.first().rank().value() as i32;
    let b = hole.second().rank().value() as i32;
    let high = a.max(b);
    let low = a.min(b);
    let suited = hole.first().suit() == hole.second().suit();
    let pair = a == b;
    let gap = high - low;
    let is_broadway = high >= 10 && low >= 10;

    match position {
        PositionBucket::HeadsUp => {
            if pair {
                return true;
            }
            if suited && high >= 2 {
                return true;
            }
            high >= 2
        }
        PositionBucket::Early => {
            if pair {
                return high >= 7;
            }
            if suited && ((high == 14 && low >= 11) || (high == 13 && low >= 12)) {
                return true; // AKs/AQs/AJs/KQs
            }
            if !suited && (high == 13 || high == 14) && low >= 12 {
                return true; // AKo/AQo/KQo
            }
            suited && gap == 1 && high >= 9
        }
        PositionBucket::Middle => {
            if pair {
                return high >= 5;
            }
            if suited && ((high == 14 && low >= 10) || (high == 13 && low >= 11)) {
                return true; // ATs+/KJs+
            }
            if suited && is_broadway {
                return true; // QJs/JTs
            }
            if !suited && (high == 13 || high == 14) && low >= 11 {
                return true; // AJo+/KJo+
            }
            suited && gap == 1 && high >= 8
        }
        PositionBucket::Late | PositionBucket::Button => {
            if pair {
                return true;
            }
            if suited && high == 14 {
                return true; // any suited ace
            }
            if suited && is_broadway {
                return true;
            }
            if !suited && high == 14 && low >= 10 {
                return true;
            }
            if !suited && high >= 11 && low >= 10 {
                return true; // KJo/QJo
            }
            suited && (gap == 1 && high >= 7 || gap == 2 && high >= 10)
        }
        PositionBucket::SmallBlind | PositionBucket::BigBlind => {
            if pair {
                return true;
            }
            if suited && high == 14 {
                return true;
            }
            if !suited && high == 14 && low >= 9 {
                return true;
            }
            if !suited && high >= 10 && low >= 10 {
                return true;
            }
            suited && (gap == 1 && high >= 6 || gap == 2 && high >= 9)
        }
    }
}

fn board_texture(board: &[Card]) -> f64 {
    if board.len() < 3 {
        return 0.0;
    }
    let mut suits = [0u8; 4];
    let mut rank_counts = [0u8; 15];
    let mut ranks: Vec<i32> = Vec::with_capacity(board.len());
    for c in board {
        let idx = match c.suit() {
            crate::cards::Suit::Clubs => 0,
            crate::cards::Suit::Diamonds => 1,
            crate::cards::Suit::Hearts => 2,
            crate::cards::Suit::Spades => 3,
        };
        suits[idx] += 1;
        let rv = c.rank().value() as usize;
        rank_counts[rv] += 1;
        ranks.push(c.rank().value() as i32);
    }
    let max_suit = *suits.iter().max().unwrap_or(&0);
    let mut texture: f64 = match max_suit {
        4..=5 => 0.5,
        3 => 0.3,
        2 => 0.15,
        _ => 0.0,
    };
    if rank_counts.iter().any(|&c| c >= 3) {
        texture += 0.25;
    } else if rank_counts.iter().any(|&c| c >= 2) {
        texture += 0.15;
    }

    ranks.sort_unstable();
    ranks.dedup();
    if ranks.contains(&14) {
        ranks.insert(0, 1);
    }
    for window in ranks.windows(4) {
        let span = window[3] - window[0];
        if span <= 4 {
            texture += 0.25;
            break;
        }
    }
    for window in ranks.windows(3) {
        let span = window[2] - window[0];
        if span <= 4 {
            texture += 0.15;
            break;
        }
    }
    texture.clamp(0.0, 1.0)
}

fn draw_bonus(hole: &HoleCards, board: &[Card], cards: &[Card]) -> f64 {
    let mut bonus = 0.0;
    let mut suits = [0u8; 4];
    for c in cards {
        let idx = match c.suit() {
            crate::cards::Suit::Clubs => 0,
            crate::cards::Suit::Diamonds => 1,
            crate::cards::Suit::Hearts => 2,
            crate::cards::Suit::Spades => 3,
        };
        suits[idx] += 1;
    }
    if suits.contains(&4) {
        bonus += 0.07;
    }
    if board.len() == 3 && suits.contains(&3) {
        let hole_suit = hole.first().suit();
        if hole.second().suit() == hole_suit {
            bonus += 0.04;
        }
    }

    let mut ranks: Vec<i32> = cards.iter().map(|c| c.rank().value() as i32).collect();
    ranks.sort_unstable();
    ranks.dedup();
    if ranks.contains(&14) {
        ranks.insert(0, 1);
    }
    for window in ranks.windows(4) {
        let span = window[3] - window[0];
        if span == 3 {
            bonus += 0.06;
            break;
        }
        if span == 4 {
            bonus += 0.03;
            break;
        }
    }

    if board.len() >= 3 {
        let max_board = board.iter().map(|c| c.rank().value()).max().unwrap_or(0);
        let ha = hole.first().rank().value();
        let hb = hole.second().rank().value();
        if ha > max_board && hb > max_board {
            bonus += 0.02;
        } else if ha > max_board || hb > max_board {
            bonus += 0.01;
        }
    }
    bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Rank, Suit};
    use crate::hand::Board;

    #[test]
    fn preflop_range_early_is_tight() {
        let aa = HoleCards::try_new(
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
        )
        .unwrap();
        let trash = HoleCards::try_new(
            Card::new(Rank::Seven, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
        )
        .unwrap();
        assert!(preflop_in_range(&aa, PositionBucket::Early));
        assert!(!preflop_in_range(&trash, PositionBucket::Early));
    }

    #[test]
    fn preflop_range_late_is_wider() {
        let suited_connector = HoleCards::try_new(
            Card::new(Rank::Eight, Suit::Spades),
            Card::new(Rank::Seven, Suit::Spades),
        )
        .unwrap();
        assert!(preflop_in_range(&suited_connector, PositionBucket::Late));
    }

    #[test]
    fn heads_up_accepts_any_ace() {
        let ace_low = HoleCards::try_new(
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
        )
        .unwrap();
        assert!(preflop_in_range(&ace_low, PositionBucket::HeadsUp));
    }

    #[test]
    fn bot_checks_or_raises_when_bet_exists() {
        let hole = HoleCards::try_new(
            Card::new(Rank::Queen, Suit::Clubs),
            Card::new(Rank::Eight, Suit::Diamonds),
        )
        .unwrap();
        let board = Board::new(Vec::new());
        let ctx = BotContext {
            seat: 0,
            dealer: 1,
            num_players: 2,
            to_call: 0,
            pot: 30,
            current_bet: 10,
            min_raise: 10,
            stack: 90,
            bet: 10,
            hole: &hole,
            board: &board,
        };
        let profile = BotProfile {
            difficulty: Difficulty::Expert,
            tightness: 0.5,
            aggression: 0.0,
            bluff: 0.0,
            tilt: 0.0,
            curiosity: 0.0,
            min_delay_ms: 0,
            max_delay_ms: 0,
            rng_seed: Some(7),
        };
        let mut state = BotState::new(profile.rng_seed);
        let decision = BotPolicy::decide(&ctx, &profile, &mut state);
        assert!(!matches!(decision.action, Action::Bet(_) | Action::BetMin));
    }
}
