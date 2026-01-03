//! Agents: pluggable players (bots, potentially humans via other frontends).
//!
//! This module introduces a small trait `PlayerAgent` and a minimal manager
//! `AgentTable` that coordinates which agent controls which seat. It lives in
//! the library so UIs (TUI/GUI) remain thin and scene logic does not need to
//! implement bot coordination.

use crate::engine::GameEngine;
use core::fmt;
use std::time::{Duration, Instant};

/// Kinds of agents attached to seats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentKind {
    Human,
    Bot,
}

/// Seat-level action intents, typically produced by a UI for a human player.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Action {
    Fold,
    CheckCall,
    BetMin,
    RaiseMin,
    Bet(u64),
    RaiseTo(u64),
}

/// A seat controller that can act for a player when it is their turn.
pub trait PlayerAgent {
    /// Called when `seat` is the current actor. Implementations may throttle internally.
    fn on_turn(
        &mut self,
        engine: &mut dyn GameEngine,
        seat: usize,
    ) -> Result<bool, crate::game::ActionError>;
    /// The kind of this agent (human, bot, etc.).
    fn kind(&self) -> AgentKind {
        AgentKind::Human
    }
    /// Optionally receive a seat-intent action; default is to ignore and return false.
    fn receive(&mut self, _action: Action) -> bool {
        false
    }
}

mod bots;

pub use bots::{BotAgent, BotConfig, BotProfile, Difficulty};

/// A simple agent that executes user-intended actions when it's their turn.
pub struct HumanAgent {
    pending: Option<Action>,
}

impl HumanAgent {
    pub fn new() -> Self {
        Self { pending: None }
    }
}

impl Default for HumanAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerAgent for HumanAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Human
    }
    fn receive(&mut self, action: Action) -> bool {
        if self.pending.is_some() {
            return false;
        }
        self.pending = Some(action);
        true
    }
    fn on_turn(
        &mut self,
        engine: &mut dyn GameEngine,
        seat: usize,
    ) -> Result<bool, crate::game::ActionError> {
        if matches!(engine.street(), crate::game::Street::Showdown) {
            self.pending = None;
            return Ok(false);
        }
        if engine.current() != seat {
            return Ok(false);
        }
        if let Some(act) = self.pending.take() {
            return match act {
                Action::Fold => engine.action_fold(),
                Action::CheckCall => engine.action_check_call(),
                Action::BetMin => engine.action_bet_min(),
                Action::RaiseMin => engine.action_raise_min(),
                Action::Bet(amount) => engine.action_bet(amount),
                Action::RaiseTo(amount) => engine.action_raise_to(amount),
            }
            .map(|_| true);
        }
        Ok(false)
    }
}

/// Manages a set of optional agents, one per seat, and drives the agent at the
/// current seat when appropriate.
pub struct AgentTable {
    seats: Vec<Option<Box<dyn PlayerAgent>>>,
    min_action_delay: Duration,
    next_action_at: Option<Instant>,
}

impl fmt::Debug for AgentTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags: Vec<char> =
            self.seats.iter().map(|a| if a.is_some() { 'B' } else { '-' }).collect();
        write!(f, "AgentTable({})", flags.into_iter().collect::<String>())
    }
}

impl AgentTable {
    /// Create a table with `n` seats, all empty.
    pub fn for_seats(n: usize) -> Self {
        let mut seats = Vec::with_capacity(n);
        for _ in 0..n {
            seats.push(None);
        }
        Self { seats, min_action_delay: Duration::from_millis(0), next_action_at: None }
    }

    /// Ensure the table has room for `n` seats.
    pub fn ensure_len(&mut self, n: usize) {
        if self.seats.len() < n {
            self.seats.resize_with(n, || None);
        }
        if self.seats.len() > n {
            self.seats.truncate(n);
        }
    }

    /// Assign an agent to a seat (or remove when `None`).
    pub fn set_agent(&mut self, seat: usize, agent: Option<Box<dyn PlayerAgent>>) {
        if seat >= self.seats.len() {
            self.ensure_len(seat + 1);
        }
        self.seats[seat] = agent;
    }

    /// Get immutable access to an agent for inspection.
    pub fn agent(&self, seat: usize) -> Option<&dyn PlayerAgent> {
        self.seats.get(seat).and_then(|a| a.as_deref())
    }

    /// Return the kind of agent at a seat, if any.
    pub fn agent_kind(&self, seat: usize) -> Option<AgentKind> {
        self.seats.get(seat).and_then(|a| a.as_deref().map(|ag| ag.kind()))
    }

    /// Send an action intent to a specific seat agent, if any.
    pub fn receive(&mut self, seat: usize, action: Action) -> bool {
        if let Some(Some(agent)) = self.seats.get_mut(seat) {
            return agent.receive(action);
        }
        false
    }

    /// Whether a seat currently has an agent assigned.
    pub fn has_agent(&self, seat: usize) -> bool {
        self.seats.get(seat).map(|a| a.is_some()).unwrap_or(false)
    }

    /// Whether any agents are currently assigned.
    pub fn any_agents(&self) -> bool {
        self.seats.iter().any(|a| a.is_some())
    }

    /// Whether any non-human (bot) agents are assigned.
    pub fn any_bots(&self) -> bool {
        self.seats.iter().filter_map(|a| a.as_deref()).any(|ag| matches!(ag.kind(), AgentKind::Bot))
    }

    /// Set a global minimum delay between any actions at the table.
    pub fn set_min_action_delay_ms(&mut self, delay_ms: u64) {
        self.min_action_delay = Duration::from_millis(delay_ms);
    }

    /// Drive the agent assigned to the current seat, if any.
    pub fn on_turn(
        &mut self,
        engine: &mut dyn GameEngine,
    ) -> Result<bool, crate::game::ActionError> {
        let seat = engine.current();
        if let Some(Some(agent)) = self.seats.get_mut(seat) {
            let is_bot = matches!(agent.kind(), AgentKind::Bot);
            let now = Instant::now();
            if is_bot {
                if let Some(next) = self.next_action_at {
                    if now < next {
                        return Ok(false);
                    }
                }
            }
            let acted = agent.on_turn(engine, seat)?;
            if acted && self.min_action_delay > Duration::from_millis(0) {
                self.next_action_at = Some(now + self.min_action_delay);
            }
            return Ok(acted);
        }
        Ok(false)
    }

    /// Remove all agents.
    pub fn clear(&mut self) {
        for a in &mut self.seats {
            *a = None;
        }
        self.next_action_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Street};
    use std::thread;
    use std::time::Duration;

    fn mk_game(n: usize) -> Game {
        Game::new(n, 1000, 5, 10)
    }

    #[test]
    fn delay_ms_throttle_actions() {
        let mut g = mk_game(3);
        g.new_hand();
        let seat = g.current;
        let mut profile = BotProfile::for_difficulty(Difficulty::Easy).with_seed(7);
        profile.min_delay_ms = 15;
        profile.max_delay_ms = 15;
        let mut bot = BotAgent::new(profile);

        g.current_bet = 0;
        g.last_raiser = None;
        g.last_raiser_acted = false;
        g.pot = 0;
        for p in &mut g.players {
            p.bet = 0;
            p.contributed = 0;
        }

        // First tick should schedule the bot and not act yet.
        let _ = bot.on_turn(&mut g, seat).unwrap();
        assert_eq!(g.current, seat, "should remain on same seat due to delay");
        assert!(g.players[seat].last_action.is_none(), "no action before delay");

        // After delay, the waiting bot should run.
        thread::sleep(Duration::from_millis(20));
        let _ = bot.on_turn(&mut g, seat).unwrap();
        assert_ne!(g.current, seat, "bot should act once delay elapsed");
    }

    #[test]
    fn showdown_noop() {
        let mut g = mk_game(3);
        g.new_hand();
        g.street = Street::Showdown; // ensure we are in terminal state
        let cur = g.current;
        let mut bot = BotAgent::new(BotProfile::default());
        let _ = bot.on_turn(&mut g, cur).unwrap();
        assert_eq!(g.current, cur, "no change at showdown");
        assert!(g.players[cur].last_action.is_none());
    }
}
