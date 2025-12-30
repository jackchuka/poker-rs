use crate::agents::{Action, AgentKind, AgentTable, BotAgent, BotProfile, Difficulty};
use crate::game::Game;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scene {
    Menu,
    Table,
}

/// High-level input actions for the TUI controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputAction {
    MenuNext,
    MenuPrev,
    MenuInc,
    MenuDec,
    MenuApply,
    MenuCancel,
    ToggleMenu,
    ToggleHelp,
    ToggleHistory,
    HistoryUp,
    HistoryDown,
    NewHand,
    Fold,
    CheckCall,
    BetMin,
    RaiseMin,
    AmountOpen,
    AmountDigit(u8),
    AmountBackspace,
    AmountIncBb,
    AmountDecBb,
    AmountSubmit,
    AmountCancel,
    BotDifficultyNext,
    FocusNext,
    FocusPrev,
    FocusSeat(usize),
}

#[derive(Debug)]
#[non_exhaustive]
pub struct AppState {
    pub scene: Scene,
    pub started: Instant,
    // Core game engine instance
    pub game: Game,
    // UI focus seat index (does not auto-move with action)
    pub focus: usize,
    pub agents: AgentTable,
    // Menu config being edited
    pub menu_index: usize,
    pub cfg_num_players: usize,
    pub cfg_starting_stack: u64,
    pub cfg_small_blind: u64,
    pub cfg_big_blind: u64,
    pub cfg_bot_delay_ms: u64,
    pub bot_delay_ms: u64,
    pub cfg_bot_difficulty: Difficulty,
    pub bot_default_difficulty: Difficulty,
    pub hand_started: bool,
    pub(crate) bot_profiles: Vec<BotProfile>,
    help_open: bool,
    history_open: bool,
    history_offset: usize,
    amount_entry: Option<String>,
    amount_entry_error: Option<String>,
    action_error: Option<String>,
    action_error_at: Option<Instant>,
}

impl Default for AppState {
    fn default() -> Self {
        let game = Game::new(5, 1000, 5, 10);
        let default_delay = 500;
        let default_difficulty = Difficulty::Medium;
        let default_profile = Self::default_bot_profile(default_delay, default_difficulty);
        Self {
            scene: Scene::Menu,
            started: Instant::now(),
            game,
            focus: 0,
            agents: AgentTable::for_seats(5),
            menu_index: 0,
            cfg_num_players: 5,
            cfg_starting_stack: 1000,
            cfg_small_blind: 5,
            cfg_big_blind: 10,
            cfg_bot_delay_ms: default_delay,
            bot_delay_ms: default_delay,
            cfg_bot_difficulty: default_difficulty,
            bot_default_difficulty: default_difficulty,
            hand_started: false,
            bot_profiles: vec![default_profile; 5],
            help_open: false,
            history_open: false,
            history_offset: 0,
            amount_entry: None,
            amount_entry_error: None,
            action_error: None,
            action_error_at: None,
        }
    }
}

impl AppState {
    pub const HISTORY_PAGE_SIZE: usize = 20;
    const ACTION_ERROR_TTL: Duration = Duration::from_secs(3);

    fn can_act_for_focus(&self) -> bool {
        if self.scene != Scene::Table || !self.hand_started {
            return false;
        }
        if self.game.players.is_empty() {
            return false;
        }
        if matches!(self.game.street, crate::game::Street::Showdown) {
            return false;
        }
        self.focus == self.game.current
    }

    fn queue_action(&mut self, action: Action) -> bool {
        if !self.can_act_for_focus() {
            return false;
        }
        self.clear_action_error();
        let _ = self.agents.receive(self.focus, action);
        true
    }

    pub fn amount_entry_active(&self) -> bool {
        self.amount_entry.is_some()
    }

    pub fn amount_entry_text(&self) -> Option<&str> {
        self.amount_entry.as_deref()
    }

    pub fn amount_entry_error(&self) -> Option<&str> {
        self.amount_entry_error.as_deref()
    }

    pub fn action_error(&self) -> Option<&str> {
        self.action_error.as_deref()
    }

    fn clear_action_error(&mut self) {
        self.action_error = None;
        self.action_error_at = None;
    }

    pub fn help_open(&self) -> bool {
        self.help_open
    }

    pub fn history_open(&self) -> bool {
        self.history_open
    }

    pub fn history_offset(&self) -> usize {
        self.history_offset
    }

    pub(crate) fn close_help(&mut self) {
        self.help_open = false;
    }

    pub(crate) fn close_history(&mut self) {
        self.history_open = false;
    }

    pub fn bot_profile_label(&self, seat: usize) -> Option<&'static str> {
        if !matches!(self.agents.agent_kind(seat), Some(AgentKind::Bot)) {
            return None;
        }
        let diff = self.bot_profiles.get(seat).map(|p| p.difficulty).unwrap_or(Difficulty::Medium);
        Some(Self::difficulty_label(diff))
    }

    pub fn difficulty_label(difficulty: Difficulty) -> &'static str {
        match difficulty {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Med",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Xprt",
        }
    }

    fn open_amount_entry(&mut self) -> bool {
        if !self.can_act_for_focus() {
            return false;
        }
        let buf = if self.game.current_bet == 0 {
            self.game.big_blind.max(1).to_string()
        } else {
            (self.game.current_bet + self.game.min_raise).to_string()
        };
        self.amount_entry = Some(buf);
        self.amount_entry_error = None;
        true
    }

    fn amount_entry_backspace(&mut self) {
        if let Some(buf) = self.amount_entry.as_mut() {
            buf.pop();
        }
        self.amount_entry_error = None;
    }

    fn amount_entry_push_digit(&mut self, digit: u8) {
        if let Some(buf) = self.amount_entry.as_mut() {
            if buf.len() >= 12 {
                return;
            }
            buf.push(char::from(b'0' + digit));
        }
        self.amount_entry_error = None;
    }

    fn amount_entry_adjust_bb(&mut self, delta: i64) {
        if let Some(buf) = self.amount_entry.as_mut() {
            let cur = buf.parse::<i64>().unwrap_or(0);
            let step = self.game.big_blind.max(1) as i64;
            let next = (cur + delta * step).max(0);
            *buf = next.to_string();
        }
        self.amount_entry_error = None;
    }

    fn amount_entry_submit(&mut self) -> bool {
        let Some(buf) = self.amount_entry.as_ref() else {
            return false;
        };
        let amount = match buf.parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                self.amount_entry_error = Some("Invalid amount".to_string());
                return false;
            }
        };
        let max_total =
            self.game.players.get(self.game.current).map(|p| p.bet + p.stack).unwrap_or(0);
        if self.game.current_bet == 0 {
            let min_bet = self.game.big_blind.max(1);
            if amount < min_bet && amount < max_total {
                self.amount_entry_error = Some(format!("Min bet is {min_bet}"));
                return false;
            }
            if self.queue_action(Action::Bet(amount)) {
                self.amount_entry = None;
                self.amount_entry_error = None;
                return true;
            }
        } else {
            let min_target = self.game.current_bet + self.game.min_raise;
            if amount < min_target && amount < max_total {
                self.amount_entry_error = Some(format!("Min raise is {min_target}"));
                return false;
            }
            if self.queue_action(Action::RaiseTo(amount)) {
                self.amount_entry = None;
                self.amount_entry_error = None;
                return true;
            }
        }
        self.amount_entry_error = Some("Action not allowed".to_string());
        false
    }

    fn amount_entry_cancel(&mut self) {
        self.amount_entry = None;
        self.amount_entry_error = None;
    }

    pub fn handle_input(&mut self, action: InputAction) -> bool {
        match action {
            InputAction::ToggleMenu => {
                self.toggle_menu();
                false
            }
            InputAction::ToggleHelp => {
                if self.scene == Scene::Table {
                    self.history_open = false;
                    self.help_open = !self.help_open;
                }
                false
            }
            InputAction::ToggleHistory => {
                if self.scene == Scene::Table {
                    self.help_open = false;
                    if !self.history_open {
                        self.history_offset = 0;
                    }
                    self.history_open = !self.history_open;
                }
                false
            }
            InputAction::HistoryUp => {
                if self.scene == Scene::Table && self.history_open {
                    let max_offset =
                        self.game.history_len().saturating_sub(Self::HISTORY_PAGE_SIZE);
                    self.history_offset = (self.history_offset + 1).min(max_offset);
                }
                false
            }
            InputAction::HistoryDown => {
                if self.scene == Scene::Table && self.history_open && self.history_offset > 0 {
                    self.history_offset -= 1;
                }
                false
            }
            InputAction::MenuNext => {
                if self.scene == Scene::Menu {
                    self.menu_next();
                }
                false
            }
            InputAction::MenuPrev => {
                if self.scene == Scene::Menu {
                    self.menu_prev();
                }
                false
            }
            InputAction::MenuInc => {
                if self.scene == Scene::Menu {
                    self.menu_inc();
                }
                false
            }
            InputAction::MenuDec => {
                if self.scene == Scene::Menu {
                    self.menu_dec();
                }
                false
            }
            InputAction::MenuApply => {
                if self.scene == Scene::Menu {
                    self.apply_menu();
                }
                false
            }
            InputAction::MenuCancel => {
                if self.scene == Scene::Menu {
                    self.cancel_menu();
                }
                false
            }
            InputAction::NewHand => {
                if self.scene == Scene::Table {
                    self.new_hand();
                }
                false
            }
            InputAction::Fold => self.queue_action(Action::Fold),
            InputAction::CheckCall => self.queue_action(Action::CheckCall),
            InputAction::BetMin => self.queue_action(Action::BetMin),
            InputAction::RaiseMin => self.queue_action(Action::RaiseMin),
            InputAction::AmountOpen => self.open_amount_entry(),
            InputAction::AmountDigit(d) => {
                self.amount_entry_push_digit(d);
                false
            }
            InputAction::AmountBackspace => {
                self.amount_entry_backspace();
                false
            }
            InputAction::AmountIncBb => {
                self.amount_entry_adjust_bb(1);
                false
            }
            InputAction::AmountDecBb => {
                self.amount_entry_adjust_bb(-1);
                false
            }
            InputAction::AmountSubmit => self.amount_entry_submit(),
            InputAction::AmountCancel => {
                self.amount_entry_cancel();
                false
            }
            InputAction::BotDifficultyNext => {
                if self.scene == Scene::Table {
                    self.cycle_focus_bot_difficulty();
                }
                false
            }
            InputAction::FocusNext => {
                if self.scene == Scene::Table {
                    self.focus_next();
                }
                false
            }
            InputAction::FocusPrev => {
                if self.scene == Scene::Table {
                    self.focus_prev();
                }
                false
            }
            InputAction::FocusSeat(idx) => {
                if self.scene == Scene::Table {
                    self.set_focus_current(idx);
                }
                false
            }
        }
    }

    pub fn new_hand(&mut self) {
        if self.hand_started && !matches!(self.game.street, crate::game::Street::Showdown) {
            return;
        }
        self.game.new_hand();
        self.hand_started = true;
        self.history_offset = 0;
        self.clear_action_error();
    }

    pub fn focus_next(&mut self) {
        if self.game.players.is_empty() {
            return;
        }
        self.focus = (self.focus + 1) % self.game.players.len();
    }

    pub fn focus_prev(&mut self) {
        if self.game.players.is_empty() {
            return;
        }
        let n = self.game.players.len();
        self.focus = (self.focus + n - 1) % n;
    }

    pub fn set_focus_current(&mut self, idx: usize) {
        if self.game.players.is_empty() {
            return;
        }
        let n = self.game.players.len();
        let i = idx % n;
        self.focus = i;
    }

    pub fn cycle_focus_bot_difficulty(&mut self) {
        if !matches!(self.agents.agent_kind(self.focus), Some(AgentKind::Bot)) {
            return;
        }
        self.ensure_bot_profiles_len(self.game.players.len());
        let current = self.bot_profiles.get(self.focus).cloned().unwrap_or_else(|| {
            Self::default_bot_profile(self.bot_delay_ms, self.bot_default_difficulty)
        });
        let next_diff = match current.difficulty {
            Difficulty::Easy => Difficulty::Medium,
            Difficulty::Medium => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Expert,
            Difficulty::Expert => Difficulty::Easy,
        };
        let mut next = BotProfile::for_difficulty(next_diff);
        next.min_delay_ms = current.min_delay_ms;
        next.max_delay_ms = current.max_delay_ms;
        next.rng_seed = current.rng_seed;
        if self.focus < self.bot_profiles.len() {
            self.bot_profiles[self.focus] = next.clone();
        }
        self.agents.set_agent(self.focus, Some(Box::new(BotAgent::new(next))));
    }

    pub fn agents_on_turn(&mut self) {
        if self.scene != Scene::Table || !self.hand_started {
            return;
        }
        if let Some(at) = self.action_error_at {
            if at.elapsed() >= Self::ACTION_ERROR_TTL {
                self.clear_action_error();
            }
        }
        self.agents.ensure_len(self.game.players.len());
        match self.agents.on_turn(&mut self.game) {
            Ok(true) => self.clear_action_error(),
            Ok(false) => {}
            Err(err) => {
                self.action_error = Some(err.to_string());
                self.action_error_at = Some(Instant::now());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_ignored_before_turn() {
        let mut app = AppState::default();
        app.apply_menu();
        app.focus = 0;
        if app.game.current == app.focus {
            app.game.current = (app.focus + 1) % app.game.players.len();
        }
        let last_action = app.game.players[app.focus].last_action.clone();

        let _ = app.queue_action(Action::Fold);
        app.game.current = app.focus;
        app.agents_on_turn();

        assert_eq!(app.game.players[app.focus].last_action, last_action);
    }
}
