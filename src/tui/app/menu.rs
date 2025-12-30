use crate::agents::{AgentTable, BotAgent, BotProfile, Difficulty, HumanAgent};
use crate::game::Game;

use super::AppState;

#[derive(Debug, Clone, Copy)]
enum MenuItem {
    Players,
    StartingStack,
    SmallBlind,
    BigBlind,
    BotDifficulty,
    BotDelayMs,
}

const MENU_ITEMS: [MenuItem; 6] = [
    MenuItem::Players,
    MenuItem::StartingStack,
    MenuItem::SmallBlind,
    MenuItem::BigBlind,
    MenuItem::BotDifficulty,
    MenuItem::BotDelayMs,
];

impl MenuItem {
    fn display(self, app: &AppState) -> String {
        match self {
            MenuItem::Players => format!("Players: {}", app.cfg_num_players),
            MenuItem::StartingStack => format!("Starting Stack: ${}", app.cfg_starting_stack),
            MenuItem::SmallBlind => format!("Small Blind: {}", app.cfg_small_blind),
            MenuItem::BigBlind => format!("Big Blind: {}", app.cfg_big_blind),
            MenuItem::BotDifficulty => {
                format!("Bot Difficulty: {}", AppState::difficulty_label(app.cfg_bot_difficulty))
            }
            MenuItem::BotDelayMs => format!("Bot Delay (ms): {}", app.cfg_bot_delay_ms),
        }
    }

    fn inc(self, app: &mut AppState) {
        match self {
            MenuItem::Players => {
                if app.cfg_num_players < 9 {
                    app.cfg_num_players += 1;
                }
            }
            MenuItem::StartingStack => {
                app.cfg_starting_stack = app.cfg_starting_stack.saturating_add(100);
            }
            MenuItem::SmallBlind => {
                app.cfg_small_blind = app.cfg_small_blind.saturating_add(1);
                if app.cfg_big_blind < app.cfg_small_blind {
                    app.cfg_big_blind = app.cfg_small_blind;
                }
            }
            MenuItem::BigBlind => {
                app.cfg_big_blind = app.cfg_big_blind.saturating_add(1);
            }
            MenuItem::BotDelayMs => {
                app.cfg_bot_delay_ms = app.cfg_bot_delay_ms.saturating_add(100);
            }
            MenuItem::BotDifficulty => {
                app.cfg_bot_difficulty = match app.cfg_bot_difficulty {
                    Difficulty::Easy => Difficulty::Medium,
                    Difficulty::Medium => Difficulty::Hard,
                    Difficulty::Hard => Difficulty::Expert,
                    Difficulty::Expert => Difficulty::Easy,
                };
            }
        }
    }

    fn dec(self, app: &mut AppState) {
        match self {
            MenuItem::Players => {
                if app.cfg_num_players > 2 {
                    app.cfg_num_players -= 1;
                }
            }
            MenuItem::StartingStack => {
                app.cfg_starting_stack = app.cfg_starting_stack.saturating_sub(100).max(100);
            }
            MenuItem::SmallBlind => {
                if app.cfg_small_blind > 1 {
                    app.cfg_small_blind -= 1;
                }
            }
            MenuItem::BigBlind => {
                if app.cfg_big_blind > 1 {
                    app.cfg_big_blind -= 1;
                    if app.cfg_big_blind < app.cfg_small_blind {
                        app.cfg_small_blind = app.cfg_big_blind;
                    }
                }
            }
            MenuItem::BotDelayMs => {
                app.cfg_bot_delay_ms = app.cfg_bot_delay_ms.saturating_sub(100);
            }
            MenuItem::BotDifficulty => {
                app.cfg_bot_difficulty = match app.cfg_bot_difficulty {
                    Difficulty::Easy => Difficulty::Expert,
                    Difficulty::Medium => Difficulty::Easy,
                    Difficulty::Hard => Difficulty::Medium,
                    Difficulty::Expert => Difficulty::Hard,
                };
            }
        }
    }
}

impl AppState {
    pub fn menu_items_display(&self) -> Vec<String> {
        MENU_ITEMS.iter().map(|item| item.display(self)).collect()
    }

    pub fn toggle_menu(&mut self) {
        self.close_help();
        self.close_history();
        self.scene = match self.scene {
            super::Scene::Menu => super::Scene::Table,
            _ => {
                self.open_menu();
                super::Scene::Menu
            }
        };
    }

    // --- Menu operations ---
    pub fn open_menu(&mut self) {
        self.close_help();
        self.close_history();
        self.menu_index = 0;
        self.cfg_num_players = self.game.players.len();
        self.cfg_starting_stack = self.game.starting_stack;
        self.cfg_small_blind = self.game.small_blind;
        self.cfg_big_blind = self.game.big_blind;
        self.cfg_bot_delay_ms = self.bot_delay_ms;
        self.cfg_bot_difficulty = self.bot_default_difficulty;
        self.scene = super::Scene::Menu;
    }

    pub fn apply_menu(&mut self) {
        // Ensure invariants
        if self.cfg_num_players < 2 {
            self.cfg_num_players = 2;
        }
        if self.cfg_small_blind == 0 {
            self.cfg_small_blind = 1;
        }
        if self.cfg_big_blind < self.cfg_small_blind {
            self.cfg_big_blind = self.cfg_small_blind;
        }

        self.bot_delay_ms = self.cfg_bot_delay_ms;
        self.bot_default_difficulty = self.cfg_bot_difficulty;
        let default_profile =
            Self::default_bot_profile(self.bot_delay_ms, self.bot_default_difficulty);
        self.bot_profiles = vec![default_profile; self.cfg_num_players];
        self.game = Game::new(
            self.cfg_num_players,
            self.cfg_starting_stack,
            self.cfg_small_blind,
            self.cfg_big_blind,
        );
        self.focus = 0;
        self.agents = AgentTable::for_seats(self.cfg_num_players);
        self.agents.set_min_action_delay_ms(150);
        self.agents.set_agent(0, Some(Box::new(HumanAgent::new())));
        for i in 1..self.cfg_num_players {
            let profile = self.bot_profiles.get(i).cloned().unwrap_or_else(|| {
                Self::default_bot_profile(self.bot_delay_ms, self.bot_default_difficulty)
            });
            self.agents.set_agent(i, Some(Box::new(BotAgent::new(profile))));
        }
        self.hand_started = false;
        self.scene = super::Scene::Table;
    }

    pub fn cancel_menu(&mut self) {
        self.scene = super::Scene::Table;
    }

    pub fn menu_next(&mut self) {
        self.menu_index = (self.menu_index + 1) % MENU_ITEMS.len();
    }
    pub fn menu_prev(&mut self) {
        self.menu_index = (self.menu_index + MENU_ITEMS.len() - 1) % MENU_ITEMS.len();
    }
    pub fn menu_inc(&mut self) {
        let item = MENU_ITEMS[self.menu_index % MENU_ITEMS.len()];
        item.inc(self);
    }
    pub fn menu_dec(&mut self) {
        let item = MENU_ITEMS[self.menu_index % MENU_ITEMS.len()];
        item.dec(self);
    }

    pub(crate) fn default_bot_profile(delay_ms: u64, difficulty: Difficulty) -> BotProfile {
        let mut profile = BotProfile::for_difficulty(difficulty);
        profile.min_delay_ms = delay_ms;
        profile.max_delay_ms = delay_ms;
        profile
    }

    pub(crate) fn ensure_bot_profiles_len(&mut self, n: usize) {
        if self.bot_profiles.len() < n {
            let profile = Self::default_bot_profile(self.bot_delay_ms, self.bot_default_difficulty);
            self.bot_profiles.resize(n, profile);
        }
        if self.bot_profiles.len() > n {
            self.bot_profiles.truncate(n);
        }
    }
}
