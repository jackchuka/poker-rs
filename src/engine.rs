// Minimal game engine API boundary. This trait exposes core poker actions and
// queries so UIs (TUI, GUI, bots) can drive the game without depending on UI
// internals. It is implemented for the core `Game` type.

pub trait GameEngine {
    // Hand lifecycle
    fn new_hand(&mut self);

    // Player actions (No-Limit Hold'em basics)
    fn action_fold(&mut self) -> Result<(), crate::game::ActionError>;
    fn action_check_call(&mut self) -> Result<(), crate::game::ActionError>;
    fn action_bet_min(&mut self) -> Result<(), crate::game::ActionError>;
    fn action_bet(&mut self, amount: u64) -> Result<(), crate::game::ActionError>;
    fn action_raise_min(&mut self) -> Result<(), crate::game::ActionError>;
    fn action_raise_to(&mut self, amount: u64) -> Result<(), crate::game::ActionError>;

    // Queries
    fn to_call(&self, seat: usize) -> u64;
    fn current_bet(&self) -> u64;
    fn min_raise(&self) -> u64;
    fn pot(&self) -> u64;
    fn hole_cards(&self, seat: usize) -> Option<crate::hand::HoleCards>;
    fn board(&self) -> &crate::hand::Board;
    fn stack(&self, seat: usize) -> u64;
    fn bet(&self, seat: usize) -> u64;
    fn current(&self) -> usize;
    fn dealer(&self) -> usize;
    fn street(&self) -> crate::game::Street;
    fn num_players(&self) -> usize;
}

impl GameEngine for crate::game::Game {
    fn new_hand(&mut self) {
        self.new_hand();
    }

    fn action_fold(&mut self) -> Result<(), crate::game::ActionError> {
        self.action_fold()
    }
    fn action_check_call(&mut self) -> Result<(), crate::game::ActionError> {
        self.action_check_call()
    }
    fn action_bet_min(&mut self) -> Result<(), crate::game::ActionError> {
        self.action_bet_min()
    }
    fn action_bet(&mut self, amount: u64) -> Result<(), crate::game::ActionError> {
        self.action_bet(amount)
    }
    fn action_raise_min(&mut self) -> Result<(), crate::game::ActionError> {
        self.action_raise_min()
    }
    fn action_raise_to(&mut self, amount: u64) -> Result<(), crate::game::ActionError> {
        self.action_raise_to(amount)
    }

    fn to_call(&self, seat: usize) -> u64 {
        self.to_call(seat)
    }
    fn current_bet(&self) -> u64 {
        self.current_bet
    }
    fn min_raise(&self) -> u64 {
        self.min_raise
    }
    fn pot(&self) -> u64 {
        self.pot
    }
    fn hole_cards(&self, seat: usize) -> Option<crate::hand::HoleCards> {
        self.players.get(seat).and_then(|p| p.hole)
    }
    fn board(&self) -> &crate::hand::Board {
        &self.board
    }
    fn stack(&self, seat: usize) -> u64 {
        self.players[seat].stack
    }
    fn bet(&self, seat: usize) -> u64 {
        self.players[seat].bet
    }
    fn current(&self) -> usize {
        self.current
    }
    fn dealer(&self) -> usize {
        self.dealer
    }
    fn street(&self) -> crate::game::Street {
        self.street
    }
    fn num_players(&self) -> usize {
        self.players.len()
    }
}
