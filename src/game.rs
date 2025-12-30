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
    pub name: String,
    pub stack: u64,
    pub bet: u64,
    pub contributed: u64,
    pub status: PlayerStatus,
    pub hole: Option<HoleCards>,
    pub last_action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PotBreakdown {
    pub(crate) main: u64,
    pub(crate) sides: Vec<u64>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Game {
    pub small_blind: u64,
    pub big_blind: u64,
    pub starting_stack: u64,

    pub deck: Deck,
    pub board: Board,
    pub players: Vec<Player>,
    pub pot: u64,
    pub dealer: usize,
    pub current: usize,
    pub street: Street,

    pub current_bet: u64,
    pub min_raise: u64,
    pub last_raiser: Option<usize>,
    pub last_raiser_acted: bool,
    pub round_starter: usize,
    pub sb_pos: Option<usize>,
    pub bb_pos: Option<usize>,
    /// Winners of the last completed hand (seat indices in table order)
    pub winners: Vec<usize>,
    /// Showdown categories for each player in the last hand (None if folded/unknown)
    pub showdown_categories: Vec<Option<Category>>,
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
        self.min_raise = self.big_blind;
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
                self.finish_showdown();
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
        while !self.is_eligible(i) {
            i = (i + 1) % n;
        }
        i
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
            self.finish_showdown();
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
            self.min_raise = self.min_raise.max(raise_amt);
            self.current_bet = new_bet;
            self.last_raiser = Some(idx);
            self.last_raiser_acted = true;
            self.round_starter = idx;
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
                    self.finish_showdown();
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

    /// Showdown: determine winners and distribute the pot (single-pot only).
    pub fn finish_showdown(&mut self) {
        let total_pot: u64 = self.players.iter().map(|p| p.contributed).sum();
        if total_pot == 0 {
            return;
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
            return;
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
            return;
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
                return;
            }
        }

        let n = self.players.len();
        let mut evals: Vec<Option<crate::evaluator::Evaluation>> = vec![None; n];
        for &i in &contenders {
            let hole = self.players[i].hole.as_ref().unwrap();
            let ev = evaluate_holdem(hole, &self.board).expect("board has 5 cards at showdown");
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
                let ev = evals[i].unwrap();
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
    }

    fn maybe_force_showdown(&mut self) {
        if matches!(self.street, Street::Showdown) {
            return;
        }
        if self.count_eligible() > 0 {
            return;
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
        self.finish_showdown();
    }

    fn record_history(&mut self, seat: usize, verb: HandHistoryVerb, amount: Option<u64>) {
        let entry = HandHistoryEntry { seat, verb, amount, street: self.street };
        self.hand_history.push(entry);
    }
}
