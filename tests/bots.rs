use poker_rs::agents::{BotAgent, BotProfile, Difficulty, PlayerAgent};
use poker_rs::game::{Game, Street};
use std::thread;
use std::time::Duration;

fn mk_game(n: usize) -> Game {
    Game::new(n, 1000, 5, 10)
}

#[test]
fn non_bot_seat_noop() {
    let mut g = mk_game(3);
    g.new_hand();
    let cur = g.current;
    let other = (cur + 1) % g.players.len();
    let mut bot = BotAgent::new(BotProfile::default());
    let _ = bot.on_turn(&mut g, other).unwrap();

    assert_eq!(g.current, cur, "current should not advance when seat isn't bot");
    assert!(g.players[cur].last_action.is_none());
}

#[test]
fn bot_acts_when_current_is_bot() {
    let mut g = mk_game(3);
    g.new_hand();
    let cur = g.current; // with 3p preflop, this is seat 1
    let mut bot = BotAgent::new(BotProfile::default());
    let _ = bot.on_turn(&mut g, cur).unwrap();

    // Action is stochastic; just assert that something happened for the current seat
    assert!(g.players[cur].last_action.is_some(), "bot should take an action");
    assert_ne!(g.current, cur, "turn should advance after bot acts");
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

#[test]
fn heads_up_preflop_bot_does_not_fold_to_blind() {
    let mut g = mk_game(2);
    g.new_hand();
    let seat = g.current;
    let mut bot = BotAgent::new(BotProfile::default());
    let _ = bot.on_turn(&mut g, seat).unwrap();

    assert!(!matches!(g.players[seat].status, poker_rs::game::PlayerStatus::Folded));
    assert_ne!(g.current, seat, "turn should advance after bot acts");
}
