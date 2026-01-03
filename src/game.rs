use crate::deck::Deck;
use crate::evaluator::{evaluate_holdem, Category};
use crate::hand::{Board, HoleCards};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerStatus {
    Active,
    Folded,
    AllIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HandHistoryVerb {
    SmallBlind,
    BigBlind,
    Fold,
    Check,
    Call,
    Bet,
    RaiseTo,
    Win,
    Split,
}

impl HandHistoryVerb {
    pub fn label(self) -> &'static str {
        match self {
            HandHistoryVerb::SmallBlind => "SB",
            HandHistoryVerb::BigBlind => "BB",
            HandHistoryVerb::Fold => "Fold",
            HandHistoryVerb::Check => "Check",
            HandHistoryVerb::Call => "Call",
            HandHistoryVerb::Bet => "Bet",
            HandHistoryVerb::RaiseTo => "Raise to",
            HandHistoryVerb::Win => "Win",
            HandHistoryVerb::Split => "Split",
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActionError {
    #[error("cannot act during showdown")]
    Showdown,
    #[error("player is not active")]
    PlayerNotActive,
    #[error("betting is not allowed when facing a bet")]
    BetNotAllowed,
    #[error("raising is not allowed without a bet")]
    RaiseNotAllowed,
    #[error("amount too small: min {min}, got {got}")]
    AmountTooSmall { min: u64, got: u64 },
    #[error("amount too large: max {max}, got {got}")]
    AmountTooLarge { max: u64, got: u64 },
    #[error("target must exceed current bet: current {current}, target {target}")]
    TargetTooLow { current: u64, target: u64 },
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShowdownError {
    #[error("hand evaluation failed: {0}")]
    EvaluationFailed(String),
    #[error("invalid game state: {0}")]
    InvalidState(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HandHistoryEntry {
    pub seat: usize,
    pub verb: HandHistoryVerb,
    pub amount: Option<u64>,
    pub street: Street,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Player {
    pub(crate) name: String,
    pub(crate) stack: u64,
    pub(crate) bet: u64,
    pub(crate) contributed: u64,
    pub(crate) status: PlayerStatus,
    pub(crate) hole: Option<HoleCards>,
    pub(crate) last_action: Option<String>,
}

impl Player {
    /// Returns the player's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the player's current stack
    pub fn stack(&self) -> u64 {
        self.stack
    }

    /// Returns the player's current bet in the current betting round
    pub fn bet(&self) -> u64 {
        self.bet
    }

    /// Returns the player's total contributed to the pot this hand
    pub fn contributed(&self) -> u64 {
        self.contributed
    }

    /// Returns the player's status
    pub fn status(&self) -> PlayerStatus {
        self.status
    }

    /// Returns the player's hole cards
    pub fn hole(&self) -> Option<HoleCards> {
        self.hole
    }

    /// Returns the player's last action as a string
    pub fn last_action(&self) -> Option<&str> {
        self.last_action.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PotBreakdown {
    pub(crate) main: u64,
    pub(crate) sides: Vec<u64>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Game {
    pub(crate) small_blind: u64,
    pub(crate) big_blind: u64,
    pub(crate) starting_stack: u64,

    pub(crate) deck: Deck,
    pub(crate) board: Board,
    pub(crate) players: Vec<Player>,
    pub(crate) pot: u64,
    pub(crate) dealer: usize,
    pub(crate) current: usize,
    pub(crate) street: Street,

    pub(crate) current_bet: u64,
    pub(crate) min_raise: u64,
    pub(crate) last_raiser: Option<usize>,
    pub(crate) last_raiser_acted: bool,
    pub(crate) round_starter: usize,
    pub(crate) sb_pos: Option<usize>,
    pub(crate) bb_pos: Option<usize>,
    /// Winners of the last completed hand (seat indices in table order)
    pub(crate) winners: Vec<usize>,
    /// Showdown categories for each player in the last hand (None if folded/unknown)
    pub(crate) showdown_categories: Vec<Option<Category>>,
    hand_history: Vec<HandHistoryEntry>,
}

impl Game {
    pub fn new(num_players: usize, starting_stack: u64, small_blind: u64, big_blind: u64) -> Self {
        let players = (1..=num_players)
            .map(|i| Player {
                name: format!("P{i}"),
                stack: starting_stack,
                bet: 0,
                contributed: 0,
                status: PlayerStatus::Active,
                hole: None,
                last_action: None,
            })
            .collect();
        Self {
            small_blind,
            big_blind,
            starting_stack,
            deck: Deck::standard(),
            board: Board::new(Vec::new()),
            players,
            pot: 0,
            dealer: 0,
            current: 0,
            street: Street::Preflop,
            current_bet: 0,
            min_raise: big_blind,
            last_raiser: None,
            last_raiser_acted: false,
            round_starter: 0,
            sb_pos: None,
            bb_pos: None,
            winners: Vec::new(),
            showdown_categories: vec![None; num_players],
            hand_history: Vec::new(),
        }
    }

    /// Returns the small blind amount
    pub fn small_blind(&self) -> u64 {
        self.small_blind
    }

    /// Returns the big blind amount
    pub fn big_blind(&self) -> u64 {
        self.big_blind
    }

    /// Returns the starting stack amount
    pub fn starting_stack(&self) -> u64 {
        self.starting_stack
    }

    /// Returns a reference to the board
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns a reference to the players
    pub fn players(&self) -> &[Player] {
        &self.players
    }

    /// Returns the current pot size
    pub fn pot(&self) -> u64 {
        self.pot
    }

    /// Returns the dealer position
    pub fn dealer(&self) -> usize {
        self.dealer
    }

    /// Returns the current player index
    pub fn current(&self) -> usize {
        self.current
    }

    /// Returns the current street
    pub fn street(&self) -> Street {
        self.street
    }

    /// Returns the current bet amount
    pub fn current_bet(&self) -> u64 {
        self.current_bet
    }

    /// Returns the minimum raise amount
    pub fn min_raise(&self) -> u64 {
        self.min_raise
    }

    /// Returns the small blind position
    pub fn sb_pos(&self) -> Option<usize> {
        self.sb_pos
    }

    /// Returns the big blind position
    pub fn bb_pos(&self) -> Option<usize> {
        self.bb_pos
    }

    /// Returns the winners of the last completed hand
    pub fn winners(&self) -> &[usize] {
        &self.winners
    }

    /// Returns the showdown categories for each player
    pub fn showdown_categories(&self) -> &[Option<Category>] {
        &self.showdown_categories
    }

    pub fn history_recent(&self, n: usize) -> Vec<HandHistoryEntry> {
        if n == 0 {
            return Vec::new();
        }
        let len = self.hand_history.len();
        let start = len.saturating_sub(n);
        self.hand_history[start..].to_vec()
    }

    pub fn history_recent_offset(&self, n: usize, offset: usize) -> Vec<HandHistoryEntry> {
        if n == 0 {
            return Vec::new();
        }
        let len = self.hand_history.len();
        if len == 0 {
            return Vec::new();
        }
        let max_offset = len.saturating_sub(n);
        let offset = offset.min(max_offset);
        let end = len.saturating_sub(offset);
        let start = end.saturating_sub(n);
        self.hand_history[start..end].to_vec()
    }

    pub fn history_len(&self) -> usize {
        self.hand_history.len()
    }

    pub fn new_hand(&mut self) {
        self.advance_dealer();
        self.reset_hand_state();
        self.reset_players_for_new_hand();
        self.align_dealer_to_eligible();
        self.winners.clear();
        self.showdown_categories = vec![None; self.players.len()];
        self.deal_hole_cards();
        self.setup_preflop();
    }

    fn advance_dealer(&mut self) {
        if !self.players.is_empty() {
            self.dealer = (self.dealer + 1) % self.players.len();
        }
    }

    fn reset_hand_state(&mut self) {
        self.deck = Deck::standard();
        let seed: u64 = rand::rng().random();
        self.deck.shuffle_seeded(seed);
        self.board = Board::new(Vec::new());
        self.pot = 0;
        self.street = Street::Preflop;
        self.hand_history.clear();
        self.current_bet = 0;
        self.min_raise = self.big_blind;
        self.last_raiser = None;
        self.last_raiser_acted = false;
        self.round_starter = self.dealer;
        self.current = self.dealer;
        self.sb_pos = None;
        self.bb_pos = None;
    }

    fn reset_players_for_new_hand(&mut self) {
        for p in &mut self.players {
            p.bet = 0;
            p.contributed = 0;
            p.hole = None;
            p.last_action = None;
            if p.stack == 0 {
                p.status = PlayerStatus::Folded;
            } else {
                p.status = PlayerStatus::Active;
            }
        }
    }

    fn align_dealer_to_eligible(&mut self) {
        if self.players.is_empty() {
            return;
        }
        let n = self.players.len();
        let mut dealer = self.dealer;
        for _ in 0..n {
            if self.is_eligible(dealer) {
                break;
            }
            dealer = (dealer + 1) % n;
        }
        self.dealer = dealer;
    }

    fn deal_hole_cards(&mut self) {
        for p in &mut self.players {
            if matches!(p.status, PlayerStatus::Active) {
                if let (Some(a), Some(b)) = (self.deck.draw(), self.deck.draw()) {
                    if let Ok(hole) = HoleCards::try_new(a, b) {
                        p.hole = Some(hole);
                    }
                }
            }
        }
    }

    fn setup_preflop(&mut self) {
        let eligible_count = self.count_eligible();
        if eligible_count < 2 {
            self.street = Street::Showdown;
            self.round_starter = self.dealer;
            self.current = self.dealer;
            self.sb_pos = None;
            self.bb_pos = None;
            return;
        }
        let (sb_pos, bb_pos) = if eligible_count == 2 {
            let sb = self.dealer;
            let bb = self.next_eligible_from(sb);
            (sb, bb)
        } else {
            let sb = self.next_eligible_from(self.dealer);
            let bb = self.next_eligible_from(sb);
            (sb, bb)
        };
        self.sb_pos = Some(sb_pos);
        self.bb_pos = Some(bb_pos);
        let bb_paid = self.post_blinds(sb_pos, bb_pos);
        self.current_bet = bb_paid;
        // Minimum raise is based on what the BB actually posted, not the nominal blind
        self.min_raise = bb_paid;
        self.last_raiser = Some(bb_pos);
        self.last_raiser_acted = false;
        if eligible_count == 2 {
            self.current = self.dealer;
        } else {
            self.current = self.next_eligible_from(bb_pos);
        }
        self.round_starter = self.current;
    }

    fn post_blinds(&mut self, sb_pos: usize, bb_pos: usize) -> u64 {
        let sb_paid = {
            let p = &mut self.players[sb_pos];
            let v = p.stack.min(self.small_blind);
            p.stack -= v;
            p.bet += v;
            p.contributed += v;
            if p.stack == 0 {
                p.status = PlayerStatus::AllIn;
            }
            p.last_action = Some(format!("SB {v}"));
            self.record_history(sb_pos, HandHistoryVerb::SmallBlind, Some(v));
            v
        };
        let bb_paid = {
            let p = &mut self.players[bb_pos];
            let v = p.stack.min(self.big_blind);
            p.stack -= v;
            p.bet += v;
            p.contributed += v;
            if p.stack == 0 {
                p.status = PlayerStatus::AllIn;
            }
            p.last_action = Some(format!("BB {v}"));
            self.record_history(bb_pos, HandHistoryVerb::BigBlind, Some(v));
            v
        };
        self.pot += sb_paid + bb_paid;
        bb_paid
    }

    pub(crate) fn pot_breakdown(&self) -> PotBreakdown {
        let mut levels: Vec<u64> =
            self.players.iter().map(|p| p.contributed).filter(|&c| c > 0).collect();
        levels.sort_unstable();
        levels.dedup();
        if levels.is_empty() {
            return PotBreakdown { main: 0, sides: Vec::new() };
        }
        let mut pots: Vec<u64> = Vec::new();
        let mut prev = 0u64;
        for lvl in levels {
            let contributors =
                self.players.iter().filter(|p| p.contributed >= lvl && p.contributed > 0).count()
                    as u64;
            if contributors == 0 {
                prev = lvl;
                continue;
            }
            let amount = (lvl - prev) * contributors;
            if amount > 0 {
                pots.push(amount);
            }
            prev = lvl;
        }
        let main = pots.first().copied().unwrap_or(0);
        let sides = if pots.len() > 1 { pots[1..].to_vec() } else { Vec::new() };
        PotBreakdown { main, sides }
    }

    fn deal_next_street(&mut self) {
        match self.street {
            Street::Preflop => {
                let drawn = self.deck.draw_n(3);
                self.board.extend(drawn);
                self.street = Street::Flop;
                self.reset_bets_set_current_postflop();
            }
            Street::Flop => {
                if let Some(c) = self.deck.draw() {
                    self.board.push(c);
                    self.street = Street::Turn;
                    self.reset_bets_set_current_postflop();
                }
            }
            Street::Turn => {
                if let Some(c) = self.deck.draw() {
                    self.board.push(c);
                    self.street = Street::River;
                    self.reset_bets_set_current_postflop();
                }
            }
            Street::River => {
                self.street = Street::Showdown;
                let _ = self.finish_showdown();
            }
            Street::Showdown => {}
        }
    }

    fn reset_bets_set_current_postflop(&mut self) {
        for p in &mut self.players {
            p.bet = 0;
            p.last_action = None;
        }
        if !self.players.is_empty() {
            let n = self.players.len();
            // Start from first seat left of dealer; skip ineligible seats
            let mut cur = (self.dealer + 1) % n;
            while !self.is_eligible(cur) {
                cur = (cur + 1) % n;
                if cur == (self.dealer + 1) % n {
                    break;
                }
            }
            self.current = cur;
            self.current_bet = 0;
            self.min_raise = self.big_blind;
            self.last_raiser = None;
            self.last_raiser_acted = false;
            self.round_starter = self.current;
        }
    }

    fn is_eligible(&self, idx: usize) -> bool {
        matches!(self.players[idx].status, PlayerStatus::Active)
    }
    fn count_eligible(&self) -> usize {
        self.players.iter().filter(|p| matches!(p.status, PlayerStatus::Active)).count()
    }
    fn next_eligible_from(&self, start: usize) -> usize {
        if self.players.is_empty() {
            return 0;
        }
        let n = self.players.len();
        let mut i = (start + 1) % n;
        for _ in 0..n {
            if self.is_eligible(i) {
                return i;
            }
            i = (i + 1) % n;
        }
        // No eligible players left; keep the cursor where it is to avoid an infinite loop.
        start % n
    }

    pub fn to_call(&self, idx: usize) -> u64 {
        if matches!(self.street, Street::Showdown) {
            return 0;
        }
        self.current_bet.saturating_sub(self.players[idx].bet)
    }

    fn ensure_can_act(&self) -> Result<(), ActionError> {
        if matches!(self.street, Street::Showdown) {
            return Err(ActionError::Showdown);
        }
        if !self.is_eligible(self.current) {
            return Err(ActionError::PlayerNotActive);
        }
        Ok(())
    }

    pub fn action_fold(&mut self) -> Result<(), ActionError> {
        self.ensure_can_act()?;
        self.players[self.current].status = PlayerStatus::Folded;
        self.players[self.current].last_action = Some("Fold".into());
        self.record_history(self.current, HandHistoryVerb::Fold, None);
        if self.count_eligible() <= 1 {
            self.street = Street::Showdown;
            let _ = self.finish_showdown();
            return Ok(());
        }
        self.advance_or_move();
        Ok(())
    }

    pub fn action_check_call(&mut self) -> Result<(), ActionError> {
        self.ensure_can_act()?;
        let to_call = self.to_call(self.current);
        if to_call == 0 {
            self.players[self.current].last_action = Some("Check".into());
            self.record_history(self.current, HandHistoryVerb::Check, None);
        } else {
            let p = &mut self.players[self.current];
            let pay = p.stack.min(to_call);
            p.stack -= pay;
            p.bet += pay;
            p.contributed += pay;
            self.pot += pay;
            if p.stack == 0 {
                p.status = PlayerStatus::AllIn;
            }
            self.players[self.current].last_action = Some(format!("Call {pay}"));
            self.record_history(self.current, HandHistoryVerb::Call, Some(pay));
        }
        self.advance_or_move();
        Ok(())
    }

    pub fn action_bet_min(&mut self) -> Result<(), ActionError> {
        self.ensure_can_act()?;
        if self.current_bet > 0 {
            return Err(ActionError::BetNotAllowed);
        }
        let target = self.big_blind.max(1);
        self.place_to_amount(target, HandHistoryVerb::Bet, "Bet")
    }

    pub fn action_bet(&mut self, amount: u64) -> Result<(), ActionError> {
        self.ensure_can_act()?;
        if self.current_bet > 0 {
            return Err(ActionError::BetNotAllowed);
        }
        let min_bet = self.big_blind.max(1);
        if amount < min_bet {
            return Err(ActionError::AmountTooSmall { min: min_bet, got: amount });
        }
        let max_total = self.players.get(self.current).map(|p| p.bet + p.stack).unwrap_or(0);
        if amount > max_total {
            return Err(ActionError::AmountTooLarge { max: max_total, got: amount });
        }
        self.place_to_amount(amount, HandHistoryVerb::Bet, "Bet")
    }

    pub fn action_raise_min(&mut self) -> Result<(), ActionError> {
        self.ensure_can_act()?;
        if self.current_bet == 0 {
            return Err(ActionError::RaiseNotAllowed);
        }
        let target = self.current_bet + self.min_raise;
        self.place_to_amount(target, HandHistoryVerb::RaiseTo, "Raise to")
    }

    pub fn action_raise_to(&mut self, amount: u64) -> Result<(), ActionError> {
        self.ensure_can_act()?;
        if self.current_bet == 0 {
            return Err(ActionError::RaiseNotAllowed);
        }
        let max_total = self.players.get(self.current).map(|p| p.bet + p.stack).unwrap_or(0);
        if amount > max_total {
            return Err(ActionError::AmountTooLarge { max: max_total, got: amount });
        }
        let min_target = self.current_bet + self.min_raise;
        if amount < min_target && amount < max_total {
            return Err(ActionError::AmountTooSmall { min: min_target, got: amount });
        }
        self.place_to_amount(amount, HandHistoryVerb::RaiseTo, "Raise to")
    }

    fn place_to_amount(
        &mut self,
        target_total: u64,
        verb: HandHistoryVerb,
        label: &str,
    ) -> Result<(), ActionError> {
        let idx = self.current;
        let curr = self.players[idx].bet;
        if target_total <= curr {
            return Err(ActionError::TargetTooLow { current: curr, target: target_total });
        }
        let need = target_total - curr;
        let new_bet = {
            let p = &mut self.players[idx];
            let pay = p.stack.min(need);
            p.stack -= pay;
            p.bet += pay;
            p.contributed += pay;
            self.pot += pay;
            if p.stack == 0 {
                p.status = PlayerStatus::AllIn;
            }
            p.last_action = Some(format!("{} {}", label, p.bet));
            p.bet
        };
        self.record_history(idx, verb, Some(new_bet));

        if new_bet > self.current_bet {
            let raise_amt = new_bet - self.current_bet;
            // Only reopen betting if this is a full raise (>= min_raise)
            // Short all-in raises don't reopen action for players who already acted
            if raise_amt >= self.min_raise {
                self.min_raise = self.min_raise.max(raise_amt);
                self.last_raiser = Some(idx);
                // Initially true: betting round can end once all active players match the bet.
                // This will be checked in should_end_round() at line 755-762.
                // Note: If the same player raises again before the round ends, advance_or_move()
                // will reset this flag to false at line 730-732, reopening action.
                self.last_raiser_acted = true;
                self.round_starter = idx;
            }
            self.current_bet = new_bet;
        }
        let prev = idx;
        self.current = self.next_eligible_from(prev);
        if self.should_end_round(prev) {
            self.deal_next_street();
        }
        self.maybe_force_showdown();
        Ok(())
    }

    fn advance_or_move(&mut self) {
        let prev = self.current;
        let next = self.next_eligible_from(prev);
        self.current = next;
        if self.should_end_round(prev) {
            match self.street {
                Street::Preflop | Street::Flop | Street::Turn => self.deal_next_street(),
                Street::River => {
                    self.street = Street::Showdown;
                    let _ = self.finish_showdown();
                }
                Street::Showdown => {}
            }
        }
        if self.last_raiser == Some(prev) && !self.last_raiser_acted {
            self.last_raiser_acted = true;
        }
        self.maybe_force_showdown();
    }

    fn should_end_round(&self, prev_actor: usize) -> bool {
        if self.count_eligible() <= 1 {
            if let Some((_, p)) = self
                .players
                .iter()
                .enumerate()
                .find(|(_, p)| matches!(p.status, PlayerStatus::Active))
            {
                if p.bet < self.current_bet {
                    return false;
                }
            }
            return true;
        }
        if self.current_bet == 0 && self.last_raiser.is_none() {
            return self.current == self.round_starter;
        }
        if let Some(lr) = self.last_raiser {
            // End when action returns to last raiser and everyone matched
            if self.last_raiser_acted {
                if self.current == lr {
                    return self
                        .players
                        .iter()
                        .filter(|p| matches!(p.status, PlayerStatus::Active))
                        .all(|p| p.bet == self.current_bet);
                }
            } else if prev_actor == lr {
                return self
                    .players
                    .iter()
                    .filter(|p| matches!(p.status, PlayerStatus::Active))
                    .all(|p| p.bet == self.current_bet);
            }
        }
        false
    }

    /// Showdown: determine winners and distribute the pot.
    ///
    /// Implements full side-pot logic to handle all-in situations correctly.
    /// Returns an error if hand evaluation fails or game state is inconsistent.
    pub fn finish_showdown(&mut self) -> Result<(), ShowdownError> {
        let total_pot: u64 = self.players.iter().map(|p| p.contributed).sum();
        if total_pot == 0 {
            return Ok(());
        }
        if self.pot != total_pot {
            self.pot = total_pot;
        }

        // Determine contenders (everyone not folded; all-in allowed). If only one, award pot.
        let contenders: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| !matches!(p.status, PlayerStatus::Folded) && p.hole.is_some())
            .map(|(i, _)| i)
            .collect();

        // Clear per-street bets; pot already has all chips.
        for p in &mut self.players {
            p.bet = 0;
        }

        if contenders.is_empty() {
            // Fallback: nobody with cards? give to UTG to avoid stuck state.
            let i =
                if self.players.is_empty() { 0 } else { (self.dealer + 1) % self.players.len() };
            let amount = self.pot;
            self.players[i].stack += amount;
            self.players[i].last_action = Some(format!("Win {amount}"));
            self.record_history(i, HandHistoryVerb::Win, Some(amount));
            self.pot = 0;
            self.winners = vec![i];
            if i < self.showdown_categories.len() {
                self.showdown_categories[i] = None;
            }
            return Ok(());
        }
        if contenders.len() == 1 {
            let i = contenders[0];
            let amount = self.pot;
            self.players[i].stack += amount;
            self.players[i].last_action = Some(format!("Win {amount}"));
            self.record_history(i, HandHistoryVerb::Win, Some(amount));
            self.pot = 0;
            self.winners = vec![i];
            if self.board.len() >= 5 {
                if let Some(h) = self.players[i].hole.as_ref() {
                    if let Ok(ev) = evaluate_holdem(h, &self.board) {
                        if i < self.showdown_categories.len() {
                            self.showdown_categories[i] = Some(ev.category);
                        }
                    }
                }
            }
            return Ok(());
        }
        if self.board.len() < 5 {
            while self.board.len() < 5 {
                if let Some(c) = self.deck.draw() {
                    self.board.push(c);
                } else {
                    break;
                }
            }
            if self.board.len() < 5 {
                let i = contenders[0];
                let amount = self.pot;
                self.players[i].stack += amount;
                self.players[i].last_action = Some(format!("Win {amount}"));
                self.record_history(i, HandHistoryVerb::Win, Some(amount));
                self.pot = 0;
                self.winners = vec![i];
                return Ok(());
            }
        }

        let n = self.players.len();
        let mut evals: Vec<Option<crate::evaluator::Evaluation>> = vec![None; n];
        for &i in &contenders {
            let hole = self.players[i].hole.as_ref().ok_or_else(|| {
                ShowdownError::InvalidState(format!("contender {i} missing hole cards"))
            })?;
            let ev = evaluate_holdem(hole, &self.board)
                .map_err(|e| ShowdownError::EvaluationFailed(format!("player {i}: {e}")))?;
            if i < self.showdown_categories.len() {
                self.showdown_categories[i] = Some(ev.category);
            }
            evals[i] = Some(ev);
        }

        let mut levels: Vec<u64> =
            self.players.iter().map(|p| p.contributed).filter(|&c| c > 0).collect();
        levels.sort_unstable();
        levels.dedup();

        let mut winnings = vec![0u64; n];
        let mut split = vec![false; n];
        let start = if n == 0 { 0 } else { (self.dealer + 1) % n };
        let mut prev = 0u64;
        for lvl in levels {
            let contributors: Vec<usize> = self
                .players
                .iter()
                .enumerate()
                .filter(|(_, p)| p.contributed >= lvl && p.contributed > 0)
                .map(|(i, _)| i)
                .collect();
            let amount = (lvl - prev) * contributors.len() as u64;
            prev = lvl;
            if amount == 0 {
                continue;
            }
            let eligible: Vec<usize> = contributors
                .iter()
                .copied()
                .filter(|&i| !matches!(self.players[i].status, PlayerStatus::Folded))
                .filter(|&i| self.players[i].hole.is_some())
                .collect();
            if eligible.is_empty() {
                continue;
            }
            let mut best = None;
            let mut pot_winners: Vec<usize> = Vec::new();
            for &i in &eligible {
                let ev = evals[i].ok_or_else(|| {
                    ShowdownError::InvalidState(format!("missing evaluation for player {i}"))
                })?;
                if let Some(b) = best {
                    if ev > b {
                        best = Some(ev);
                        pot_winners.clear();
                        pot_winners.push(i);
                    } else if ev == b {
                        pot_winners.push(i);
                    }
                } else {
                    best = Some(ev);
                    pot_winners.push(i);
                }
            }
            if pot_winners.is_empty() {
                continue;
            }
            pot_winners.sort_by_key(|&i| (i + n - start) % n);
            let per = amount / pot_winners.len() as u64;
            let mut rem = (amount % pot_winners.len() as u64) as usize;
            for &i in &pot_winners {
                let mut amt = per;
                if rem > 0 {
                    amt += 1;
                    rem -= 1;
                }
                winnings[i] = winnings[i].saturating_add(amt);
                if pot_winners.len() > 1 {
                    split[i] = true;
                }
            }
        }

        let mut winners: Vec<usize> = Vec::new();
        for i in 0..n {
            let amt = winnings[i];
            if amt == 0 {
                continue;
            }
            self.players[i].stack += amt;
            self.players[i].last_action =
                Some(if split[i] { format!("Split {amt}") } else { format!("Win {amt}") });
            let verb = if split[i] { HandHistoryVerb::Split } else { HandHistoryVerb::Win };
            self.record_history(i, verb, Some(amt));
            winners.push(i);
        }

        winners.sort_by_key(|&i| (i + n - start) % n);
        self.pot = 0;
        // Reset betting state to neutral values after the hand ends
        self.current_bet = 0;
        self.min_raise = self.big_blind;
        self.last_raiser = None;
        self.last_raiser_acted = false;
        self.round_starter = self.current;
        self.winners = winners;
        Ok(())
    }

    fn maybe_force_showdown(&mut self) {
        if matches!(self.street, Street::Showdown) {
            return;
        }
        let active = self.count_eligible();
        if active > 1 {
            return;
        }
        if active == 1 {
            if let Some((_, p)) = self
                .players
                .iter()
                .enumerate()
                .find(|(_, p)| matches!(p.status, PlayerStatus::Active))
            {
                if p.bet < self.current_bet {
                    return;
                }
            }
        }
        let contenders = self
            .players
            .iter()
            .filter(|p| !matches!(p.status, PlayerStatus::Folded) && p.hole.is_some())
            .count();
        if contenders > 1 && self.board.len() < 5 {
            while self.board.len() < 5 {
                if let Some(c) = self.deck.draw() {
                    self.board.push(c);
                } else {
                    break;
                }
            }
        }
        self.street = Street::Showdown;
        let _ = self.finish_showdown();
    }

    fn record_history(&mut self, seat: usize, verb: HandHistoryVerb, amount: Option<u64>) {
        let entry = HandHistoryEntry { seat, verb, amount, street: self.street };
        self.hand_history.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Rank, Suit};
    use crate::hand::{Board, HoleCards};

    fn mk_game(n: usize) -> Game {
        Game::new(n, 1000, 5, 10)
    }

    fn hole(a: Card, b: Card) -> HoleCards {
        HoleCards::try_new(a, b).expect("valid hole cards")
    }

    #[test]
    fn side_pots_distribute_across_all_in_levels() {
        let mut g = mk_game(3);
        g.street = Street::Showdown;
        g.board = Board::new(vec![
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Diamonds),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Eight, Suit::Spades),
            Card::new(Rank::King, Suit::Clubs),
        ]);

        g.players[0].hole =
            Some(hole(Card::new(Rank::Queen, Suit::Spades), Card::new(Rank::Queen, Suit::Hearts)));
        g.players[1].hole =
            Some(hole(Card::new(Rank::Ace, Suit::Spades), Card::new(Rank::Ace, Suit::Hearts)));
        g.players[2].hole =
            Some(hole(Card::new(Rank::Seven, Suit::Clubs), Card::new(Rank::Six, Suit::Clubs)));

        g.players[0].status = PlayerStatus::AllIn;
        g.players[1].status = PlayerStatus::AllIn;
        g.players[2].status = PlayerStatus::AllIn;

        g.players[0].contributed = 100;
        g.players[1].contributed = 50;
        g.players[2].contributed = 200;
        g.pot = 350;

        g.players[0].stack = 0;
        g.players[1].stack = 0;
        g.players[2].stack = 0;

        let _ = g.finish_showdown();

        assert_eq!(g.players[1].stack, 150, "main pot should go to best hand");
        assert_eq!(g.players[0].stack, 100, "side pot should go to next best hand");
        assert_eq!(g.players[2].stack, 100, "single-eligible side pot goes to contributor");
    }

    #[test]
    fn split_main_pot_and_single_side_pot() {
        let mut g = mk_game(3);
        g.street = Street::Showdown;
        g.board = Board::new(vec![
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Queen, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Two, Suit::Clubs),
        ]);

        g.players[0].hole =
            Some(hole(Card::new(Rank::Ten, Suit::Clubs), Card::new(Rank::Three, Suit::Diamonds)));
        g.players[1].hole =
            Some(hole(Card::new(Rank::Ten, Suit::Hearts), Card::new(Rank::Four, Suit::Spades)));
        g.players[2].hole =
            Some(hole(Card::new(Rank::Nine, Suit::Clubs), Card::new(Rank::Nine, Suit::Diamonds)));

        for p in &mut g.players {
            p.status = PlayerStatus::AllIn;
            p.stack = 0;
        }

        g.players[0].contributed = 50;
        g.players[1].contributed = 50;
        g.players[2].contributed = 200;
        g.pot = 300;

        let _ = g.finish_showdown();

        assert_eq!(g.players[0].stack, 75, "main pot split between tied winners");
        assert_eq!(g.players[1].stack, 75, "main pot split between tied winners");
        assert_eq!(g.players[2].stack, 150, "side pot goes to lone contributor");
    }

    #[test]
    fn split_main_and_side_pots() {
        let mut g = mk_game(4);
        g.street = Street::Showdown;
        g.board = Board::new(vec![
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Queen, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Two, Suit::Clubs),
        ]);

        g.players[0].hole =
            Some(hole(Card::new(Rank::Ten, Suit::Clubs), Card::new(Rank::Three, Suit::Diamonds)));
        g.players[1].hole =
            Some(hole(Card::new(Rank::Ten, Suit::Hearts), Card::new(Rank::Four, Suit::Spades)));
        g.players[2].hole =
            Some(hole(Card::new(Rank::Nine, Suit::Clubs), Card::new(Rank::Nine, Suit::Diamonds)));
        g.players[3].hole =
            Some(hole(Card::new(Rank::Nine, Suit::Hearts), Card::new(Rank::Nine, Suit::Spades)));

        for p in &mut g.players {
            p.status = PlayerStatus::AllIn;
            p.stack = 0;
        }

        g.players[0].contributed = 50;
        g.players[1].contributed = 50;
        g.players[2].contributed = 100;
        g.players[3].contributed = 100;
        g.pot = 300;

        let _ = g.finish_showdown();

        assert_eq!(g.players[0].stack, 100, "main pot split between tied winners");
        assert_eq!(g.players[1].stack, 100, "main pot split between tied winners");
        assert_eq!(g.players[2].stack, 50, "side pot split between tied winners");
        assert_eq!(g.players[3].stack, 50, "side pot split between tied winners");
    }

    #[test]
    fn odd_chip_split_uses_seat_order() {
        let mut g = mk_game(3);
        g.street = Street::Showdown;
        g.dealer = 0;
        g.board = Board::new(vec![
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Queen, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Two, Suit::Clubs),
        ]);

        g.players[0].hole =
            Some(hole(Card::new(Rank::Ten, Suit::Clubs), Card::new(Rank::Three, Suit::Diamonds)));
        g.players[1].hole =
            Some(hole(Card::new(Rank::Ten, Suit::Hearts), Card::new(Rank::Four, Suit::Spades)));
        g.players[2].hole =
            Some(hole(Card::new(Rank::Nine, Suit::Clubs), Card::new(Rank::Nine, Suit::Diamonds)));

        for p in &mut g.players {
            p.status = PlayerStatus::AllIn;
            p.stack = 0;
        }

        g.players[0].contributed = 1;
        g.players[1].contributed = 1;
        g.players[2].contributed = 2;
        g.pot = 4;

        let _ = g.finish_showdown();

        assert_eq!(g.players[0].stack, 1, "tie loser should receive smaller share");
        assert_eq!(g.players[1].stack, 2, "odd chip awarded by seat order");
        assert_eq!(g.players[2].stack, 1, "single-eligible side pot still awarded");
    }

    #[test]
    fn showdown_deals_remaining_board_cards() {
        let mut game = Game::new(3, 100, 5, 10);
        game.new_hand();

        for p in &mut game.players {
            p.status = PlayerStatus::Folded;
        }
        game.players[0].status = PlayerStatus::AllIn;
        game.players[1].status = PlayerStatus::AllIn;
        game.players[0].contributed = 50;
        game.players[1].contributed = 50;
        game.pot = 100;
        game.board = Board::new(Vec::new());
        game.street = Street::Showdown;

        let _ = game.finish_showdown();

        assert_eq!(game.board.len(), 5);
        assert!(game.pot == 0);
        assert!(!game.winners.is_empty());
    }

    #[test]
    fn zero_stack_players_sit_out_next_hand() {
        let mut game = Game::new(3, 100, 5, 10);
        game.players[1].stack = 0;

        game.new_hand();

        let busted = &game.players[1];
        assert!(matches!(busted.status, PlayerStatus::Folded));
        assert!(busted.hole.is_none());
        assert_eq!(busted.bet, 0);
        assert_eq!(busted.contributed, 0);
        assert_ne!(game.current, 1);
    }

    #[test]
    fn postflop_bet_and_calls_advance() {
        let mut game = Game::new(3, 1000, 5, 10);
        game.new_hand();
        // Skip preflop
        for _ in 0..3 {
            game.action_check_call().unwrap();
        }
        assert_eq!(game.street, Street::Flop);
        game.action_bet(20).unwrap();
        game.action_check_call().unwrap();
        game.action_check_call().unwrap();
        assert_eq!(game.street, Street::Turn);
    }

    #[test]
    fn auto_showdown_when_all_players_all_in() {
        let mut game = Game::new(3, 100, 5, 10);
        game.new_hand();
        let utg = game.current;
        let sb = (game.dealer + 1) % game.players.len();
        let bb = (game.dealer + 2) % game.players.len();
        game.players[utg].stack = 200;
        game.players[sb].stack = 10;
        game.players[bb].stack = 10;
        game.action_raise_to(200).unwrap();
        game.action_check_call().unwrap();
        game.action_check_call().unwrap();
        assert_eq!(game.street, Street::Showdown);
    }

    #[test]
    fn auto_showdown_when_one_active_and_others_all_in() {
        let mut game = Game::new(3, 200, 5, 10);
        game.new_hand();

        let sb = game.sb_pos.expect("sb set");
        let bb = game.bb_pos.expect("bb set");
        let utg = game.current;

        game.players[utg].stack = 200;
        game.players[sb].stack = 10;
        game.players[bb].stack = 10;

        let raise_to = game.current_bet + 40;
        game.action_raise_to(raise_to).unwrap();
        game.action_check_call().unwrap();
        game.action_check_call().unwrap();

        assert!(matches!(game.street, Street::Showdown));
        assert_eq!(game.board.len(), 5);
    }
}
